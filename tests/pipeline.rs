use actor::mailbox::{simple, stop, MultiSender, Receiver, Sender};
use actor::Actor;
use async_trait::async_trait;
use tokio::time;

const FLUSH_INTERVAL: time::Duration = time::Duration::from_millis(100);
const MAX_BYTES_IN_FLIGHT: usize = 15;

/// Representation of the commitment of bytes up until this position.
#[derive(Debug, Clone, Copy)]
struct BytesCommitment(usize);

impl BytesCommitment {
    fn update(self, later: Self) -> Self {
        later
    }
}

struct Tailer<I, O, C>
where
    I: Receiver<u8>,
    O: Sender<(Vec<u8>, BytesCommitment)>,
    C: Receiver<BytesCommitment>,
{
    /// Input that we are tailing
    input: I,

    /// Buffered output
    output: O,

    /// Commitments of buffers.
    commitments: C,

    /// Stop signal
    stop: stop::Stop,

    /// current buffer of not-yet-sent bytes
    buf: Vec<u8>,

    /// Time at which this buffer should be flushed
    flush_at: Option<time::Instant>,

    /// total number of bytes read
    read: usize,

    /// total number of bytes committed at the end of the pipeline
    committed: usize,
}

impl<I, O, C> Tailer<I, O, C>
where
    I: Receiver<u8>,
    O: Sender<(Vec<u8>, BytesCommitment)>,
    C: Receiver<BytesCommitment>,
{
    fn new(input: I, output: O, commitments: C, stop: stop::Stop) -> Self {
        Tailer {
            input,
            output,
            commitments,
            stop,
            buf: vec![],
            flush_at: None,
            read: 0,
            committed: 0,
        }
    }

    fn read_byte(&mut self, c: u8) {
        self.read += 1;
        if self.flush_at.is_none() {
            self.flush_at = Some(time::Instant::now() + FLUSH_INTERVAL);
        }
        self.buf.push(c);
    }

    async fn flush(&mut self) {
        if self.buf.len() > 0 {
            let buf = std::mem::take(&mut self.buf);
            self.flush_at = None;
            self.output
                .send((buf, BytesCommitment(self.read)))
                .await
                .unwrap();
        }
    }
}

#[async_trait]
impl<I, O, C> Actor for Tailer<I, O, C>
where
    I: Receiver<u8>,
    O: Sender<(Vec<u8>, BytesCommitment)>,
    C: Receiver<BytesCommitment>,
{
    async fn run(mut self) {
        let mut stopping = false;
        loop {
            // time::sleep(..) call is evaluated even if the condition
            // is false, so we must provide an actual non-negative duration.
            let now = time::Instant::now();
            let wake_at = self
                .flush_at
                .unwrap_or(time::Instant::now() + time::Duration::from_millis(100));
            let until_flush = if wake_at < now {
                time::Duration::ZERO
            } else {
                wake_at - now
            };
            tokio::select! {
                _ = time::sleep(until_flush), if self.flush_at.is_some() => {
                    self.flush().await;
                },
                rx = self.commitments.recv() => {
                    if let Some(comm) = rx {
                        self.committed = comm.0;
                    } else {
                        panic!("downstream actor failed");
                    }
                },
                rx = self.input.recv(), if !stopping && self.read - self.committed < MAX_BYTES_IN_FLIGHT => {
                    if let Some(c) = rx {
                        self.read_byte(c);
                    } else {
                        self.flush().await;
                        // stop if the input channel closes
                        stopping = true;
                    }
                },
                _ = self.stop.recv(), if !stopping => {
                    // stop if requested (even if the input channel remains open)
                    stopping = true;
                }
            }
            if stopping && self.committed == self.read {
                break;
            }
        }
    }
}

struct Consumer<I, C>
where
    I: Receiver<(Vec<u8>, BytesCommitment)>,
    C: MultiSender<BytesCommitment>,
{
    input: I,
    commits: C,
}

#[async_trait]
impl<I, C> Actor for Consumer<I, C>
where
    I: Receiver<(Vec<u8>, BytesCommitment)>,
    C: MultiSender<BytesCommitment>,
{
    async fn run(mut self) {
        loop {
            tokio::select! {
                rx = self.input.recv() => {
                    if let Some((v, comm)) = rx {
                        println!("GOT {:?}", v);
                        let sender = self.commits.clone();
                        tokio::spawn(async move {
                            time::sleep(time::Duration::from_millis(300)).await;
                            println!("ACK {:?}", v);
                            sender.send(comm).await.unwrap();
                        });
                    } else {
                        break
                    }
                },
            }
        }
    }
}

#[tokio::test]
async fn producer_consumer() {
    let (out_bytes, in_bytes) = simple::new();
    let (out_bufs, in_bufs) = simple::new();
    let (mut stopper, stop) = stop::new();
    let (out_comms, in_comms) = simple::new();
    let mut t = Tailer::spawn(Tailer::new(in_bytes, out_bufs, in_comms, stop));
    let mut c = Consumer::spawn(Consumer {
        input: in_bufs,
        commits: out_comms,
    });

    for b in b"abcdefghijklmnopqrabcdefghijklmnopqrabcdefghijklmnopqrsssabcdefghijklmnopqrs" {
        out_bytes.send(*b).await.unwrap();
        time::sleep(time::Duration::from_millis(10)).await;
    }

    stopper.stop();

    t.stopped().await.unwrap();
    c.stopped().await.unwrap();
}
