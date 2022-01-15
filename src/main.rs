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

struct Main;

#[async_trait]
impl Actor for Main {
    async fn run(self) {
        let (outbox, inbox) = Mailbox::new().split();
        let mut p = Producer::spawn(Producer {
            outbox,
            data: vec![1, 2, 3, 4],
        });
        let mut c = Consumer::spawn(Consumer { inbox });

        p.stopped().await.unwrap();
        c.stopped().await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let mut m = Main::spawn(Main);
    m.stopped().await.unwrap();
}
