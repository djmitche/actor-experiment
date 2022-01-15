use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Mailbox<T: Sync + Send + 'static> {
    tx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>,
}

impl<T: Sync + Send + 'static> Mailbox<T> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1);
        Mailbox { tx, rx }
    }

    pub fn split(self) -> (Outbox<T>, Inbox<T>) {
        (Outbox { tx: self.tx }, Inbox { rx: self.rx })
    }
}

#[derive(Debug, Clone)]
pub struct Outbox<T: Sync + Send + 'static> {
    pub tx: mpsc::Sender<T>,
}

#[derive(Debug)]
pub struct Inbox<T: Sync + Send + 'static> {
    pub rx: mpsc::Receiver<T>,
}
