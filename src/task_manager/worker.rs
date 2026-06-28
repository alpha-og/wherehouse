use std::sync::mpsc::Sender;
use std::thread;

pub struct Worker {
    tx: Sender<bool>,
    pub thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new<F>(tx: Sender<bool>, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let thread = thread::spawn(f);
        Self { tx, thread }
    }

    pub fn stop(&self) -> color_eyre::Result<()> {
        match self.tx.send(true) {
            _ => Ok(()),
        }
    }
}
