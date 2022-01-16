use actor::mailbox::{simple, Receiver, Sender};
use actor::Actor;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

struct Producer<Tx: Sender<u8>> {
    outbox: Tx,
    data: Vec<u8>,
}

#[async_trait]
impl<Tx: Sender<u8>> Actor for Producer<Tx> {
    async fn run(self) {
        for byte in self.data {
            self.outbox.send(byte).await.unwrap()
        }
    }
}

// ---

struct Consumer<Rx: Receiver<u8>> {
    inbox: Rx,
    result: Arc<Mutex<Vec<Option<u8>>>>,
}

#[async_trait]
impl<Rx: Receiver<u8>> Actor for Consumer<Rx> {
    async fn run(mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.inbox.recv() => {
                    self.result.lock().unwrap().push(Some(msg));
                }
                else => {
                    self.result.lock().unwrap().push(None);
                    break;
                }
            }
        }
    }
}

// ---

struct Main;

#[async_trait]
impl Actor for Main {
    async fn run(self) {
        let result = Arc::new(Mutex::new(Vec::new()));
        let (outbox, inbox) = simple::new();
        let mut p = Producer {
            outbox,
            data: vec![1, 2, 3, 4],
        }
        .spawn();
        let mut c = Consumer {
            inbox,
            result: result.clone(),
        }
        .spawn();

        p.stopped().await.unwrap();
        c.stopped().await.unwrap();

        assert_eq!(
            result.lock().unwrap().clone(),
            vec![Some(1), Some(2), Some(3), Some(4), None]
        );
    }
}

#[tokio::test]
async fn producer_consumer() {
    let mut m = Main::spawn(Main);
    m.stopped().await.unwrap();
}
