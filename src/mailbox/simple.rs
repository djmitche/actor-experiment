use crate::mailbox;
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Create a new simple mailbox, represented as a sender and a receiver.
///
/// The sender can be cloned.  Once all senders have been dropped, the receiver will receiv None.
pub fn new<T: std::fmt::Debug + Sync + Send + 'static>(
) -> (impl mailbox::MultiSender<T>, impl mailbox::Receiver<T>) {
    let (tx, rx) = mpsc::channel(1);
    (Sender { tx }, Receiver { rx })
}

#[derive(Debug)]
struct Sender<T: Sync + Send + 'static> {
    tx: mpsc::Sender<T>,
}

#[async_trait]
impl<T: std::fmt::Debug + Sync + Send + 'static> mailbox::Sender<T> for Sender<T> {
    async fn send(&self, t: T) -> anyhow::Result<()> {
        Ok(self.tx.send(t).await?)
    }
}

impl<T: std::fmt::Debug + Sync + Send + 'static> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

#[derive(Debug)]
struct Receiver<T: Sync + Send + 'static> {
    rx: mpsc::Receiver<T>,
}

#[async_trait]
impl<T: std::fmt::Debug + Sync + Send + 'static> mailbox::Receiver<T> for Receiver<T> {
    async fn recv(&mut self) -> Option<T> {
        self.rx.recv().await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mailbox::{Receiver, Sender};

    #[tokio::test]
    async fn send_stuff() -> anyhow::Result<()> {
        let (tx, mut rx) = new();
        tokio::spawn(async move {
            tx.send("hello").await.unwrap();
            tx.send("world").await.unwrap();
        });

        assert_eq!(rx.recv().await.unwrap(), "hello");
        assert_eq!(rx.recv().await.unwrap(), "world");
        assert_eq!(rx.recv().await, None);

        Ok(())
    }
}
