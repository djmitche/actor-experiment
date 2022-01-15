use tokio::sync::oneshot;

#[derive(Debug)]
pub struct Oneshot<T: Sync + Send + 'static> {
    tx: oneshot::Sender<T>,
    rx: oneshot::Receiver<T>,
}

impl<T: Sync + Send + 'static> Oneshot<T> {
    pub fn new() -> Self {
        let (tx, rx) = oneshot::channel();
        Oneshot { tx, rx }
    }

    pub fn split(self) -> (OneshotOutbox<T>, OneshotInbox<T>) {
        (OneshotOutbox { tx: self.tx }, OneshotInbox { rx: self.rx })
    }
}

#[derive(Debug)]
pub struct OneshotOutbox<T: Sync + Send + 'static> {
    pub tx: oneshot::Sender<T>,
}

#[derive(Debug)]
pub struct OneshotInbox<T: Sync + Send + 'static> {
    pub rx: oneshot::Receiver<T>,
}
