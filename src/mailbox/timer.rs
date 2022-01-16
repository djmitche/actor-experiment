use std::future::{pending, ready};
use tokio::time;

pub fn new() -> Timer {
    Timer { at: None }
}

/// A Timer allows an actor to set a time at which it wishes to be notified.
/// This is useful for flushing buffers.
pub struct Timer {
    at: Option<time::Instant>,
}

impl Timer {
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
