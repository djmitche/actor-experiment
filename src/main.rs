#![allow(dead_code)]
use async_trait::async_trait;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

/**
 * Thinking:
 *
 *  - an actor _framework_ is hard, esp without dictating an async runtime
 *  - an actor _toolbox_ is different
 *    - pluggable bits: mailboxes, monitoring, telemetry
 *    - patterns: how to write a loop, initialization, ..
 *    - designs: when to use one mailbox or multiple, etc.
 *    - encoded with types, traits, or macros where possible
 *  - shippable as lots of crates, with easy addition by others
 *  - mostly focus on mailboxes
 */

#[derive(Debug)]
struct Mailbox<T: Sync + Send + 'static> {
    tx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>,
}

impl<T: Sync + Send + 'static> Mailbox<T> {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel(1);
        Mailbox { tx, rx }
    }

    fn split(self) -> (Outbox<T>, Inbox<T>) {
        (Outbox { tx: self.tx }, Inbox { rx: self.rx })
    }
}

#[derive(Debug, Clone)]
struct Outbox<T: Sync + Send + 'static> {
    tx: mpsc::Sender<T>,
}

#[derive(Debug)]
struct Inbox<T: Sync + Send + 'static> {
    rx: mpsc::Receiver<T>,
}

// ---

#[derive(Debug)]
struct Monitor {
    children: HashMap<String, mpsc::Receiver<()>>,
}

impl Monitor {
    fn new() -> Self {
        Monitor {
            children: HashMap::new(),
        }
    }

    fn child(&mut self, name: impl Into<String>) -> ChildMonitor {
        let (tx, rx) = mpsc::channel(1);
        self.children.insert(name.into(), rx);
        ChildMonitor { tx }
    }

    async fn wait(&mut self, name: impl AsRef<str>) {
        let child = self.children.get_mut(name.as_ref()).unwrap();
        child.recv().await;
    }
}

struct ChildMonitor {
    tx: mpsc::Sender<()>,
}

// ---

#[derive(Debug)]
struct Oneshot<T: Sync + Send + 'static> {
    tx: oneshot::Sender<T>,
    rx: oneshot::Receiver<T>,
}

impl<T: Sync + Send + 'static> Oneshot<T> {
    fn new() -> Self {
        let (tx, rx) = oneshot::channel();
        Oneshot { tx, rx }
    }

    fn split(self) -> (OneshotOutbox<T>, OneshotInbox<T>) {
        (OneshotOutbox { tx: self.tx }, OneshotInbox { rx: self.rx })
    }
}

#[derive(Debug)]
struct OneshotOutbox<T: Sync + Send + 'static> {
    tx: oneshot::Sender<T>,
}

#[derive(Debug)]
struct OneshotInbox<T: Sync + Send + 'static> {
    rx: oneshot::Receiver<T>,
}

// ---

struct ProducerOptions {
    mon: ChildMonitor,
    outbox: Outbox<u8>,
    data: Vec<u8>,
}

struct Producer {
    mon: ChildMonitor,
    outbox: Outbox<u8>,
    data: Vec<u8>,
}

impl Producer {
    async fn spawn(options: ProducerOptions) {
        let a = Producer {
            mon: options.mon,
            outbox: options.outbox,
            data: options.data,
        };
        tokio::spawn(async move { a.run().await });
    }

    async fn run(self) {
        for byte in self.data {
            self.outbox.tx.send(byte).await.unwrap()
        }
    }
}

// ---

struct ConsumerOptions {
    mon: ChildMonitor,
    inbox: Inbox<u8>,
}

struct Consumer {
    mon: ChildMonitor,
    inbox: Inbox<u8>,
}

impl Consumer {
    async fn spawn(options: ConsumerOptions) {
        let a = Consumer {
            mon: options.mon,
            inbox: options.inbox,
        };
        tokio::spawn(async move { a.run().await });
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.inbox.rx.recv() => {
                    println!("GOT {}", msg);
                }
                else => {
                    println!("consumer complete");
                    break;
                }
            }
        }
    }
}

// ---

struct MainOptions {
    done: OneshotOutbox<()>,
}

struct Main {
    done: OneshotOutbox<()>,
}

impl Main {
    fn spawn(options: MainOptions) {
        let a = Main { done: options.done };
        tokio::spawn(async move { a.run().await });
    }

    async fn run(self) {
        let mut monitor = Monitor::new();
        let (outbox, inbox) = Mailbox::new().split();
        Producer::spawn(ProducerOptions {
            mon: monitor.child("producer"),
            outbox,
            data: vec![1, 2, 3, 4],
        })
        .await;
        Consumer::spawn(ConsumerOptions {
            mon: monitor.child("consumer"),
            inbox,
        })
        .await;

        monitor.wait("producer").await;
        monitor.wait("consumer").await;

        self.done.tx.send(()).unwrap();
    }
}

#[tokio::main]
async fn main() {
    let (outbox, inbox) = Oneshot::new().split();
    Main::spawn(MainOptions { done: outbox });
    inbox.rx.await.unwrap();
}
