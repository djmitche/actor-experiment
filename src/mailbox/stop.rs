use tokio::sync::oneshot;

/// A Stopper provides a way to send a stop signal to a single actor.
///
/// # Example
///
/// ```
/// # use tokio::time::{sleep, Duration};
/// # use actor::mailbox::Stopper;
/// # #[tokio::test]
/// # async fn select_stop() {
/// let (mut stopper, mut stop) = Stopper::new();
/// let task = tokio::spawn(async move {
///     let mut stopping = false;
///     loop {
///         tokio::select! {
///             // when stopped, set a flag and do not call stop.recv()
///             // again.
///             _ = stop.recv(), if !stopping => {
///                 stopping = true;
///             },
///             _ = sleep(Duration::from_millis(1)) => {
///                 // complete some other work before stopping..
///                 if stopping {
///                     break
///                 }
///             },
///         }
///     }
/// });
/// stopper.stop();
/// task.await.unwrap();
/// # }
/// ```
#[derive(Debug)]
pub struct Stopper {
    tx: Option<oneshot::Sender<()>>,
}

impl Stopper {
    pub fn new() -> (Stopper, Stop) {
        let (tx, rx) = oneshot::channel();
        (Stopper { tx: Some(tx) }, Stop { rx })
    }

    /// Send a stop signal to the actor.  Calls after the first do nothing.  If the actor has
    /// already stopped, nothing happens.
    pub fn stop(&mut self) {
        if let Some(tx) = self.tx.take() {
            // an error here means the Receiver has been dropped, and can be ignored.
            let _ = tx.send(());
        }
    }
}

/// Stop is the receiver counterpart to [`Stopper`].
#[derive(Debug)]
pub struct Stop {
    rx: oneshot::Receiver<()>,
}

impl Stop {
    /// Wait for a stop signal.  Once the stop signal has been sent, this must not be polled again.
    /// If the Stopper is dropped, that is treated as a stop signal.
    pub async fn recv(&mut self) {
        // an error here means the sender has been dropped, which we treat
        // as equivalen to sending a stop signal.
        let _ = (&mut self.rx).await;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn start_and_stop() {
        let (mut stopper, mut stop) = Stopper::new();
        let task = tokio::spawn(async move { stop.recv().await });
        stopper.stop();
        task.await.unwrap();
    }

    #[tokio::test]
    async fn stop_twice() {
        let (mut stopper, mut stop) = Stopper::new();
        let task = tokio::spawn(async move { stop.recv().await });
        stopper.stop();
        stopper.stop();
        task.await.unwrap();
    }

    #[tokio::test]
    async fn stopper_dropped() {
        let (stopper, mut stop) = Stopper::new();
        let task = tokio::spawn(async move { stop.recv().await });
        drop(stopper);
        task.await.unwrap();
    }
}
