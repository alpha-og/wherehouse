use crate::{
    commands::{self, CommandType},
    state::State,
};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        mpsc::{self, Sender},
    },
    thread,
};

pub enum TaskEvent {
    Stop,
}

pub struct TaskManager {
    state: Arc<State>,
    pool: HashMap<CommandType, Worker>,
}

impl TaskManager {
    pub fn new(state: Arc<State>) -> Self {
        Self {
            state,
            pool: HashMap::default(),
        }
    }

    pub fn execute(&mut self, command_type: CommandType) -> color_eyre::Result<()> {
        match command_type {
            CommandType::Search => {
                let state = self.state.clone();
                let (tx_task, rx_task) = mpsc::channel::<TaskEvent>();
                let worker = Worker::new(tx_task, move || {
                    let search = state.search.lock().unwrap();
                    let query = search.query.clone();
                    let source = search.source;
                    drop(search);
                    let search_results = commands::search(rx_task, query, source);
                    let mut search = state.search.lock().unwrap();

                    if let Some(results) = search_results {
                        search.results = results;
                    } else {
                        search.results = Vec::default();
                    }
                });
                if let Some(worker) = self.pool.insert(CommandType::Search, worker) {
                    worker.stop()?;
                }
            }
            _ => {}
        };
        Ok(())
    }
}

struct Worker {
    tx: Sender<TaskEvent>,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new<F>(tx: Sender<TaskEvent>, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let thread = thread::spawn(f);
        Self { tx, thread }
    }

    pub fn stop(&self) -> color_eyre::Result<()> {
        match self.tx.send(TaskEvent::Stop) {
            _ => Ok(()), // Ok(_) => Ok(()),
                         // Err(e) => bail!("an error occurred while stopping the thread: {e}"),
        }
    }
}
