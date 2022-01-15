use ::actor::*;
use async_trait::async_trait;
use tokio::time;

const FLUSH_INTERVAL: time::Duration = time::Duration::from_millis(100);

struct Tailer {
    input: Inbox<u8>,
    output: Outbox<Vec<u8>>,

    /// current buffer of not-yet-sent bytes
    buf: Vec<u8>,

    /// Time at which this buffer should be flushed
    flush_at: Option<time::Instant>,

    /// total number of bytes read
    read: usize,

    /// total number of bytes committed at the end of the pipeline
    committed: usize,
}

impl Tailer {
    fn new(input: Inbox<u8>, output: Outbox<Vec<u8>>) -> Self {
        Tailer {
            input,
            output,
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
            self.output.tx.send(buf).await.unwrap();
        }
    }
}

#[async_trait]
impl Actor for Tailer {
    async fn run(mut self) {
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
                rx = self.input.rx.recv() => {
                    if let Some(c) = rx {
                        self.read_byte(c);
                    } else {
                        self.flush().await;
                        break;
                    }
                },
            }
        }
    }
}

struct Consumer {
    input: Inbox<Vec<u8>>,
}

#[async_trait]
impl Actor for Consumer {
    async fn run(mut self) {
        loop {
            tokio::select! {
                rx = self.input.rx.recv() => {
                    if let Some(v) = rx {
                        println!("GOT {:?}", v);
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
    let (out_bytes, in_bytes) = Mailbox::new().split();
    let (out_bufs, in_bufs) = Mailbox::new().split();
    let mut t = Tailer::spawn(Tailer::new(in_bytes, out_bufs));
    let mut c = Consumer::spawn(Consumer { input: in_bufs });

    for b in b"abcdefghijklmnopqrabcdefghijklmnopqrabcdefghijklmnopqrsssabcdefghijklmnopqrs" {
        out_bytes.tx.send(*b).await.unwrap();
        time::sleep(time::Duration::from_millis(10)).await;
    }

    time::sleep(time::Duration::from_secs(3)).await;
    drop(out_bytes);

    t.stopped().await.unwrap();
    c.stopped().await.unwrap();

    assert!(false);
}
