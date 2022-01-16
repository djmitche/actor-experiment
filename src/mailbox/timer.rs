use std::future::{pending, ready};
use tokio::time;

/// A Timer allows an actor to set a time at which it wishes to be notified.
///
/// # Examples
///
/// ```
/// # use actor::mailbox::{simple, Receiver, Sender, Timer};
/// # use actor::Actor;
/// # use tokio::time::{Instant, Duration};
/// # use async_trait::async_trait;
/// # use std::sync::{Arc, Mutex};
/// struct Batcher<Rx: Receiver<&'static str>, Tx: Sender<String>> {
///     buf: String,
///     flush_timer: Timer,
///     rx: Rx,
///     tx: Tx,
/// }
/// #[async_trait]
/// impl<Rx: Receiver<&'static str>, Tx: Sender<String>> Actor for Batcher<Rx, Tx> {
///     async fn run(mut self) {
///         loop {
///             tokio::select! {
///                 Some(msg) = self.rx.recv() => {
///                     if self.buf.len() == 0 {
///                         self.flush_timer.set(Instant::now() + Duration::from_millis(100));
///                     }
///                     self.buf.push_str(msg);
///                 },
///                 _ = self.flush_timer.recv(), if self.buf.len() > 0 => {
///                     let buf = std::mem::take(&mut self.buf);
///                     self.tx.send(buf).await;
///                 }
///             }
///         }
///     }
/// }
///
/// #[tokio::test]
/// async fn producer_consumer() {
///     let (in_tx, in_rx) = simple::new();
///     let (out_tx, out_rx) = simple::new();
///     let mut b = Batcher {
///         buf: String::new(),
///         flush_timer: Timer::new(),
///         rx: in_rx,
///         tx: out_tx,
///     }.spawn();
///
///     in_tx.send("hello ").await.unwrap();
///     in_tx.send("world").await.unwrap();
///
///     let result = out_rx.recv().await;
///     assert_eq!(result.unwrap(), String::from("hello world"));
/// }
/// ```
pub struct Timer {
    at: Option<time::Instant>,
}

impl Timer {
    /// Create a new, cleared Timer.
    pub fn new() -> Timer {
        Timer { at: None }
    }

    /// Set the timer to expire at the given time
    pub fn set(&mut self, at: time::Instant) {
        self.at = Some(at);
    }

    /// Clear the timer.  While clear, a timer will not receive anything.
    pub fn clear(&mut self) {
        self.at = None;
    }

    /// Wait for the timer to expire.
    pub async fn recv(&mut self) {
        if let Some(at) = self.at {
            let now = time::Instant::now();
            if at > now {
                time::sleep(at - now).await;
            } else {
                ready(()).await
            }
        } else {
            pending::<()>().await;
        }
    }
}
