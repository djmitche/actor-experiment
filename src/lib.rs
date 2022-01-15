use async_trait::async_trait;
use tokio::sync::watch;

pub mod mailbox;

#[derive(Debug, Clone)]
pub struct Handle {
    finish_watch: watch::Receiver<bool>,
}

impl Handle {
    pub async fn stopped(&mut self) -> anyhow::Result<()> {
        while !*self.finish_watch.borrow() {
            self.finish_watch.changed().await?;
        }
        Ok(())
    }
}

#[async_trait]
pub trait Actor: Send + Sized + 'static {
    fn spawn(self) -> Handle {
        // this watch channel watches for the child task to terminate
        // TODO: test that this handles panics
        let (tx, rx) = watch::channel(false);
        tokio::spawn(async move {
            self.run().await;
            let _ = tx.send(true); // fails if no recievers
        });
        Handle { finish_watch: rx }
    }

    async fn run(self);
}
