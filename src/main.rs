#![allow(dead_code)]
#![allow(unused_imports)]
use async_trait::async_trait;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

mod actor;
mod mailbox;

use actor::*;
use mailbox::*;

// ---

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

// ---

struct Producer {
    mon: ChildMonitor,
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
    mon: ChildMonitor,
    inbox: Inbox<u8>,
}

#[async_trait]
impl Actor for Consumer {
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

struct Main {
    done: OneshotOutbox<()>,
}

#[async_trait]
impl Actor for Main {
    async fn run(self) {
        let mut monitor = Monitor::new();
        let (outbox, inbox) = Mailbox::new().split();
        Producer::spawn(Producer {
            mon: monitor.child("producer"),
            outbox,
            data: vec![1, 2, 3, 4],
        });
        Consumer::spawn(Consumer {
            mon: monitor.child("consumer"),
            inbox,
        });

        monitor.wait("producer").await;
        monitor.wait("consumer").await;

        self.done.tx.send(()).unwrap();
    }
}

#[tokio::main]
async fn main() {
    let (outbox, inbox) = Oneshot::new().split();
    Main::spawn(Main { done: outbox });
    inbox.rx.await.unwrap();
}
