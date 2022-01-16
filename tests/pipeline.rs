use actor::actor::map;
use actor::mailbox::{simple, MultiSender, Receiver, Sender, Stop, Stopper, Timer};
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
    stop: Stop,

    /// Timer for flushing
    flush_timer: Timer,

    /// current buffer of not-yet-sent bytes
    buf: Vec<u8>,

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
    fn new(input: I, output: O, commitments: C, stop: Stop) -> Self {
        Tailer {
            input,
            output,
            commitments,
            stop,
            flush_timer: Timer::new(),
            buf: vec![],
            read: 0,
            committed: 0,
        }
    }

    fn read_byte(&mut self, c: u8) {
        self.read += 1;
        if self.buf.len() == 0 {
            self.flush_timer.set(time::Instant::now() + FLUSH_INTERVAL);
        }
        self.buf.push(c);
    }

    async fn flush(&mut self) {
        if self.buf.len() > 0 {
            let buf = std::mem::take(&mut self.buf);
            self.flush_timer.clear();
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
            tokio::select! {
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
                _ = self.flush_timer.recv(), if self.buf.len() > 0 => {
                    self.flush().await;
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
    I: Receiver<(String, BytesCommitment)>,
    C: MultiSender<BytesCommitment>,
{
    input: I,
    commits: C,
}

#[async_trait]
impl<I, C> Actor for Consumer<I, C>
where
    I: Receiver<(String, BytesCommitment)>,
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
    let (out_bytes, in_bytes) = simple();
    let (out_bufs, in_bufs) = simple();
    let (out_strs, in_strs) = simple();
    let (mut stopper, stop) = Stopper::new();
    let (out_comms, in_comms) = simple();
    let mut t = Tailer::new(in_bytes, out_bufs, in_comms, stop).spawn();
    let mut ts = map(in_bufs, out_strs, |(buf, comm)| {
        (String::from_utf8(buf).unwrap(), comm)
    });
    let mut c = Consumer {
        input: in_strs,
        commits: out_comms,
    }
    .spawn();

    for b in b"abcdefghijklmnopqrabcdefghijklmnopqrabcdefghijklmnopqrsssabcdefghijklmnopqrs" {
        out_bytes.send(*b).await.unwrap();
        time::sleep(time::Duration::from_millis(10)).await;
    }

    stopper.stop();

    t.stopped().await.unwrap();
    ts.stopped().await.unwrap();
    c.stopped().await.unwrap();
}
