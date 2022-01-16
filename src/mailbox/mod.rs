use async_trait::async_trait;

pub mod simple;
pub mod stop;
pub mod timer;

/// A Receiver allows consumption of incoming messages.  Typically a receiver will be used as input
/// by a single actor.
#[async_trait]
pub trait Receiver<T: Sync + Send + 'static>: Send + Sync + 'static {
    /// Receive a message from the mailbox.  If the mailbox is unable to produce more messages,
    /// this returns None.
    async fn recv(&mut self) -> Option<T>;
}

/// A MultiReceiver is a Receiver that can be cloned.  When multiple receivers exist, a message in
/// the mailbox will be received by exactly one receiver.
#[async_trait]
pub trait MultiReceiver<T: Sync + Send + 'static>: Clone + Receiver<T> {}

impl<T, S> MultiReceiver<T> for S
where
    T: Send + Sync + 'static,
    S: Receiver<T> + Clone,
{
}

/// A Sender allows sending messages. Typically a sender will be used
/// by one or more actors to send messages to another actor.
#[async_trait]
pub trait Sender<T: Sync + Send + 'static>: Send + Sync + 'static {
    /// Send a message to the mailbox.  This may block, depending on the semantics
    /// of the underlying mailbox.
    async fn send(&self, t: T) -> anyhow::Result<()>;
}

/// A MultiSender is a Sender that can be cloned.  All clones of a MultiSender
/// send to the same mailbox.
#[async_trait]
pub trait MultiSender<T: Sync + Send + 'static>: Clone + Sender<T> {}

impl<T, S> MultiSender<T> for S
where
    T: Send + Sync + 'static,
    S: Sender<T> + Clone,
{
}
