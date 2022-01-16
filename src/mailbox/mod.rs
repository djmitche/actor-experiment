//! Maiboxes provide for communication between actors.
//!
//! Most maiboxes have two "halves", similar to a channel or a pipe.  The sender
//! and receiver halves are typically distributed to different actors to allow
//! communication between those actors.  Some mailboxes allow cloning the senders
//! and/or receivers to support MPSC or even MPMC operation.  Finally, some
//! mailboxes only have a receiver.  These are typically used within a single
//! actor to manage timers or other internal events.
//!
//! # Examples
//!
//! ```
//! # use actor::mailbox::{simple, Receiver, Sender};
//! # use actor::Actor;
//! # use async_trait::async_trait;
//! struct Producer<Tx: Sender<&'static str>>(Tx);
//! #[async_trait]
//! impl<Tx: Sender<&'static str>> Actor for Producer<Tx> {
//!     async fn run(self) {
//!         self.0.send("hello").await.unwrap();
//!         self.0.send("world").await.unwrap();
//!     }
//! }
//!
//! struct Consumer<Rx: Receiver<&'static str>>(Rx);
//! #[async_trait]
//! impl<Rx: Receiver<&'static str>> Actor for Consumer<Rx> {
//!     async fn run(mut self) {
//!         assert_eq!(self.0.recv().await, Some("hello"));
//!         assert_eq!(self.0.recv().await, Some("world"));
//!         assert_eq!(self.0.recv().await, None);
//!     }
//! }
//!
//! #[tokio::test]
//! async fn producer_consumer() {
//!     // create a new mailbox
//!     let (sender, receiver) = simple::new();
//!     // distribute the halves of that mailbox to two actors
//!     let mut p = Producer(sender);
//!     let mut c = Consumer(receiver);
//!     // wait until they complete and check that they did not fail
//!     p.stopped().await.unwrap();
//!     c.stopped().await.unwrap();
//! }
//! ```
use async_trait::async_trait;

mod simple;
mod stop;
mod timer;

pub use simple::simple;
pub use stop::{Stop, Stopper};
pub use timer::Timer;

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
