use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Monitor {
    children: HashMap<String, mpsc::Receiver<()>>,
}

impl Monitor {
    pub fn new() -> Self {
        Monitor {
            children: HashMap::new(),
        }
    }

    pub fn child(&mut self, name: impl Into<String>) -> ChildMonitor {
        let (tx, rx) = mpsc::channel(1);
        self.children.insert(name.into(), rx);
        ChildMonitor { tx }
    }

    pub async fn wait(&mut self, name: impl AsRef<str>) {
        let child = self.children.get_mut(name.as_ref()).unwrap();
        child.recv().await;
    }
}

pub struct ChildMonitor {
    pub tx: mpsc::Sender<()>,
}
