use tracing::info;
use wherehouse::package_manager::{self, Command, PackageManager, homebrew::Homebrew};

use crate::state::{Pane, State};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        mpsc::{self, Sender},
    },
    thread,
};

pub struct TaskManager<T> {
    state: Arc<State>,
    package_manager: Arc<T>,
    pool: HashMap<Command, Worker>,
}

impl<T: PackageManager + Send + Sync + 'static> TaskManager<T> {
    pub fn new(state: Arc<State>, package_manager: Arc<T>) -> Self {
        Self {
            state,
            package_manager,
            pool: HashMap::default(),
        }
    }

    pub fn execute(&mut self, command: Command) -> color_eyre::Result<()> {
        let state = self.state.clone();
        let package_manager = self.package_manager.clone();
        let (tx_task, rx_task) = mpsc::channel::<bool>();

        let worker = match command {
            Command::FilterPackages => Worker::new(tx_task, move || {
                let search = state.search();
                let query = search.query.clone();
                info!("Command::FilterPackages => {query}");
                let source = search.source;
                drop(search);
                if query.is_empty() {
                    let mut search = state.search.lock().unwrap();
                    search.results = Vec::default();
                    return;
                }
                let result = package_manager.filter_packages(rx_task, source, query);
                let mut search = state.search.lock().unwrap();

                let output = match result {
                    Ok(results) => results,
                    Err(_) => Vec::default(),
                };
                search.results = output;
            }),
            Command::PackageInfo => Worker::new(tx_task, move || {
                let search = state.search();
                let package_name = match search.list_state.selected() {
                    Some(selected) => match search.results.get(selected) {
                        Some(result) => result.clone(),
                        None => return,
                    },
                    None => return,
                };
                info!("Command::PackageInfo => {package_name}");
                drop(search);
                let result = package_manager.package_info(rx_task, package_name);
                let mut search = state.search.lock().unwrap();
                let output = match result {
                    Ok(output) => output,
                    Err(_) => String::default(),
                };

                state.set_current_pane(Pane::SearchResults(output.clone()));
                search.selected_result_info = output;
            }),
            Command::InstallPackage => Worker::new(tx_task, move || {
                let search = state.search();
                let package_name = match search.list_state.selected() {
                    Some(selected) => match search.results.get(selected) {
                        Some(result) => result.clone(),
                        None => return,
                    },
                    None => return,
                };
                info!("Command::InstallPackage => {package_name}");
                drop(search);
                let result = package_manager.install_package(rx_task, package_name);
                let output = match result {
                    Ok(output) => output,
                    Err(_) => String::default(),
                };

                state.set_current_pane(Pane::SearchResults(output.clone()));
            }),
            Command::UninstallPackage => Worker::new(tx_task, move || {
                let search = state.search();
                let package_name = match search.list_state.selected() {
                    Some(selected) => match search.results.get(selected) {
                        Some(result) => result.clone(),
                        None => return,
                    },
                    None => return,
                };
                drop(search);
                let result = package_manager.uninstall_package(rx_task, package_name);
                let output = match result {
                    Ok(output) => output,
                    Err(_) => String::default(),
                };

                state.set_current_pane(Pane::SearchResults(output.clone()));
            }),

            Command::CheckHealth => Worker::new(tx_task, move || {
                let result = package_manager.check_health(rx_task);
                let output = match result {
                    Ok(output) => output,
                    Err(_) => String::default(),
                };

                state.set_current_pane(Pane::About(output.clone()));
                state.set_healthcheck_results(output);
            }),
            Command::Config => Worker::new(tx_task, move || {
                let result = package_manager.package_manager_config(rx_task);
                let mut config = state.config.lock().unwrap();
                let output = match result {
                    Ok(output) => output,
                    Err(_) => String::default(),
                };
                state.set_current_pane(Pane::About(output.clone()));
                config.system_config = output;
            }),
            _ => Worker::new(tx_task, || {}),
        };
        if let Some(worker) = self.pool.insert(command, worker) {
            worker.stop()?;
        }

        Ok(())
    }
}

struct Worker {
    tx: Sender<bool>,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new<F>(tx: Sender<bool>, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let thread = thread::spawn(f);
        Self { tx, thread }
    }

    pub fn stop(&self) -> color_eyre::Result<()> {
        match self.tx.send(true) {
            _ => Ok(()), // Ok(_) => Ok(()),
                         // Err(e) => bail!("an error occurred while stopping the thread: {e}"),
        }
    }
}
