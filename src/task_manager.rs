use tracing::info;

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

    pub fn execute(
        &mut self,
        command_type: CommandType,
        update_context: bool,
    ) -> color_eyre::Result<()> {
        let state = self.state.clone();
        let (tx_task, rx_task) = mpsc::channel::<TaskEvent>();

        let worker = match command_type {
            CommandType::Search => Worker::new(tx_task, move || {
                let search = state.search.lock().unwrap();
                let query = search.query.clone();
                let source = search.source;
                drop(search);
                let result = commands::search(rx_task, query, source);
                let mut search = state.search.lock().unwrap();

                let output = match result {
                    Some(results) => results,
                    None => Vec::default(),
                };
                search.results = output;
            }),
            CommandType::Info => Worker::new(tx_task, move || {
                let search = state.search.lock().unwrap();
                let package_name = match search.results.get(search.selected_result) {
                    Some(result) => result.display_text.clone(),
                    None => String::default(),
                };
                drop(search);
                let result = commands::info(rx_task, package_name);
                let mut search = state.search.lock().unwrap();
                let output = match result {
                    Some(output) => output,
                    None => String::default(),
                };
                if update_context {
                    state.update_context(output.clone());
                }

                search.selected_result_info = output;
            }),
            CommandType::Healthcheck => Worker::new(tx_task, move || {
                let result = commands::check_health(rx_task);
                let mut healthcheck_results = state.healthcheck_results.lock().unwrap();
                let output = match result {
                    Some(output) => output,
                    None => String::default(),
                };
                if update_context {
                    state.update_context(output.clone());
                }

                *healthcheck_results = output;
            }),
            CommandType::Config => Worker::new(tx_task, move || {
                let result = commands::config(rx_task);
                let mut config = state.config.lock().unwrap();
                let output = match result {
                    Some(output) => output,
                    None => String::default(),
                };
                if update_context {
                    state.update_context(output.clone());
                }
                config.system_config = output;
            }),
            _ => Worker::new(tx_task, || {}),
        };
        if let Some(worker) = self.pool.insert(command_type, worker) {
            worker.stop()?;
        }

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
