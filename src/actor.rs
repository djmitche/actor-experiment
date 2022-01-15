use async_trait::async_trait;
use std::sync::Arc;

struct HandleInner {
    task: tokio::task::JoinHandle<()>,
}

pub struct Handle(Arc<HandleInner>);

impl Handle {
    // TODO: implement wait, or be able to wrap this into a mailbox that implements it
}

#[async_trait]
pub trait Actor: Send + Sized + 'static {
    fn spawn(self) -> Handle {
        let task = tokio::spawn(async move { self.run().await });
        Handle(Arc::new(HandleInner { task: task }))
    }

    async fn run(self);
}
