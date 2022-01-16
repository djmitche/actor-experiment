use crate::mailbox::{Receiver, Sender};
use crate::{Actor, Handle};
use async_trait::async_trait;
use std::marker::PhantomData;

/// Start an actor that receives input of type I, transforms it to type O, and
/// writes it to a Sender.  The actor stops when its input is closed.
///
/// # Examples
///
/// ```
/// # use actor::mailbox::{simple, Receiver, Sender};
/// # use actor::actor::map;
///
/// #[tokio::test]
/// async fn producer_consumer() {
///     // mailboxes for input and output
///     let (in_tx, in_rx) = simple::new();
///     let (out_tx, out_rx) = simple::new();
///     let mut m = map(in_rx, out_tx, |str| { str.parse().unwrap() });
///
///     in_tx.send("123").await.unwrap();
///     assert_eq!(out_rx.recv().await, Some(123));
///     in_tx.send("456").await.unwrap();
///     assert_eq!(out_rx.recv().await, Some(456));
///     drop(in_tx);
///     assert_eq!(out_rx.recv().await, None);
/// }
/// ```
pub fn map<I, Rx, O, Tx, F>(input: Rx, output: Tx, func: F) -> Handle
where
    I: std::fmt::Debug + Sync + Send + 'static,
    Rx: Receiver<I>,
    O: std::fmt::Debug + Sync + Send + 'static,
    Tx: Sender<O>,
    F: Fn(I) -> O + Sync + Send + 'static,
{
    Map {
        input,
        output,
        func,
        _io: PhantomData,
    }
    .spawn()
}

struct Map<I, Rx, O, Tx, F>
where
    I: std::fmt::Debug + Sync + Send + 'static,
    Rx: Receiver<I>,
    O: std::fmt::Debug + Sync + Send + 'static,
    Tx: Sender<O>,
    F: Fn(I) -> O + Sync + Send + 'static,
{
    input: Rx,
    output: Tx,
    func: F,
    _io: PhantomData<(I, O)>,
}

#[async_trait]
impl<I, Rx, O, Tx, F> Actor for Map<I, Rx, O, Tx, F>
where
    I: std::fmt::Debug + Sync + Send + 'static,
    Rx: Receiver<I>,
    O: std::fmt::Debug + Sync + Send + 'static,
    Tx: Sender<O>,
    F: Fn(I) -> O + Sync + Send + 'static,
{
    async fn run(mut self) {
        while let Some(msg) = self.input.recv().await {
            self.output.send((self.func)(msg)).await.unwrap();
        }
    }
}
