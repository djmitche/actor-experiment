use actor::mailbox::{Inbox, Outbox, Simple};
use actor::Actor;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

struct Producer {
    outbox: Outbox<u8>,
    data: Vec<u8>,
}

#[async_trait]
impl Actor for Producer {
    async fn run(self) {
        for byte in self.data {
            self.outbox.tx.send(byte).await.unwrap()
        }
    }
}

// ---

struct Consumer {
    inbox: Inbox<u8>,
    result: Arc<Mutex<Vec<Option<u8>>>>,
}

#[async_trait]
impl Actor for Consumer {
    async fn run(mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.inbox.rx.recv() => {
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
        let (outbox, inbox) = Simple::new().split();
        let mut p = Producer::spawn(Producer {
            outbox,
            data: vec![1, 2, 3, 4],
        });
        let mut c = Consumer::spawn(Consumer {
            inbox,
            result: result.clone(),
        });

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
