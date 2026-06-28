use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, mpsc};

use tracing::info;
use wherehouse::package_manager::{Command, PackageManager};

use crate::state::Event;
use crate::state::State;
use super::worker::Worker;

pub struct TaskManager {
    state: Arc<State>,
    backends: Arc<Vec<Arc<dyn PackageManager>>>,
    tx: Sender<Event>,
    pool: HashMap<Command, Worker>,
}

impl TaskManager {
    pub fn new(state: Arc<State>, backends: Arc<Vec<Arc<dyn PackageManager>>>, tx: Sender<Event>) -> Self {
        Self {
            state,
            backends,
            tx,
            pool: HashMap::default(),
        }
    }

    pub fn execute(&mut self, command: Command) -> color_eyre::Result<()> {
        let state = self.state.clone();
        let backends = self.backends.clone();
        let tx = self.tx.clone();
        let (tx_task, rx_task) = mpsc::channel::<bool>();

        let worker = match command {
            Command::FilterPackages => Worker::new(tx_task, move || {
                let search = state.search();
                let query = search.query.clone();
                drop(search);
                info!("Command::FilterPackages => {query}");
                let pm = backends[state.current_backend_index()].clone();
                let result = pm.filter_packages(rx_task, query.clone());
                match result {
                    Ok((results, warning)) => {
                        let _ = tx.send(Event::SearchCompleted { results, warning, query });
                    }
                    Err(e) => {
                        let _ = tx.send(Event::SearchCompleted {
                            results: Vec::new(),
                            warning: Some(format!("Search failed: {e}")),
                            query,
                        });
                    }
                }
            }),
            Command::PackageInfo => Worker::new(tx_task, move || {
                let search = state.search();
                let package_name = match search.list_state.selected() {
                    Some(selected) => match search.results.get(selected) {
                        Some(result) => result.name.clone(),
                        None => return,
                    },
                    None => return,
                };
                info!("Command::PackageInfo => {package_name}");
                drop(search);
                let pm = backends[state.current_backend_index()].clone();
                let result = pm.package_info(rx_task, package_name);
                let output = match result {
                    Ok(output) => output,
                    Err(e) => {
                        let _ = tx.send(Event::CommandFailed {
                            cmd: Command::PackageInfo,
                            error: e.to_string(),
                        });
                        return;
                    }
                };
                let _ = tx.send(Event::CommandOutputReceived {
                    cmd: Command::PackageInfo,
                    output,
                });
            }),
            Command::InstallPackage => Worker::new(tx_task, move || {
                let search = state.search();
                let package_name = match search.list_state.selected() {
                    Some(selected) => match search.results.get(selected) {
                        Some(result) => result.name.clone(),
                        None => return,
                    },
                    None => return,
                };
                info!("Command::InstallPackage => {package_name}");
                drop(search);
                let pm = backends[state.current_backend_index()].clone();
                let result = pm.install_package(rx_task, package_name);
                let output = match result {
                    Ok(output) => output,
                    Err(_) => String::default(),
                };
                let _ = tx.send(Event::CommandOutputReceived {
                    cmd: Command::InstallPackage,
                    output,
                });
            }),
            Command::UninstallPackage => Worker::new(tx_task, move || {
                let search = state.search();
                let package_name = match search.list_state.selected() {
                    Some(selected) => match search.results.get(selected) {
                        Some(result) => result.name.clone(),
                        None => return,
                    },
                    None => return,
                };
                info!("Command::UninstallPackage => {package_name}");
                drop(search);
                let pm = backends[state.current_backend_index()].clone();
                let result = pm.uninstall_package(rx_task, package_name);
                let output = match result {
                    Ok(output) => output,
                    Err(_) => String::default(),
                };
                let _ = tx.send(Event::CommandOutputReceived {
                    cmd: Command::UninstallPackage,
                    output,
                });
            }),
            Command::CheckHealth => Worker::new(tx_task, move || {
                let pm = backends[state.current_backend_index()].clone();
                let result = pm.check_health(rx_task);
                let output = match result {
                    Ok(output) => output,
                    Err(e) => {
                        let _ = tx.send(Event::CommandFailed {
                            cmd: Command::CheckHealth,
                            error: e.to_string(),
                        });
                        return;
                    }
                };
                let _ = tx.send(Event::CommandOutputReceived {
                    cmd: Command::CheckHealth,
                    output,
                });
            }),
            Command::UpdatePackage => Worker::new(tx_task, move || {
                let search = state.search();
                let package_name = match search.list_state.selected() {
                    Some(selected) => match search.results.get(selected) {
                        Some(result) => result.name.clone(),
                        None => return,
                    },
                    None => return,
                };
                info!("Command::UpdatePackage => {package_name}");
                drop(search);
                let pm = backends[state.current_backend_index()].clone();
                let result = pm.update_package(rx_task, package_name);
                let output = match result {
                    Ok(output) => output,
                    Err(_) => String::default(),
                };
                let _ = tx.send(Event::CommandOutputReceived {
                    cmd: Command::UpdatePackage,
                    output,
                });
            }),
            Command::UpdateAll => Worker::new(tx_task, move || {
                info!("Command::UpdateAll");
                let pm = backends[state.current_backend_index()].clone();
                let result = pm.update_all_packages(rx_task);
                let output = match result {
                    Ok(output) => output,
                    Err(_) => String::default(),
                };
                let _ = tx.send(Event::CommandOutputReceived {
                    cmd: Command::UpdateAll,
                    output,
                });
            }),
            Command::CheckOutdated => Worker::new(tx_task, move || {
                info!("Command::CheckOutdated");
                let pm = backends[state.current_backend_index()].clone();
                let result = pm.check_outdated(rx_task);
                let outdated = match result {
                    Ok(names) => names,
                    Err(_) => Vec::new(),
                };
                let _ = tx.send(Event::OutdatedCheckCompleted { outdated });
            }),
            Command::Config => Worker::new(tx_task, move || {
                let pm = backends[state.current_backend_index()].clone();
                let result = pm.package_manager_config(rx_task);
                let output = match result {
                    Ok(output) => output,
                    Err(e) => {
                        let _ = tx.send(Event::CommandFailed {
                            cmd: Command::Config,
                            error: e.to_string(),
                        });
                        return;
                    }
                };
                let _ = tx.send(Event::CommandOutputReceived {
                    cmd: Command::Config,
                    output,
                });
            }),
            _ => Worker::new(tx_task, || {}),
        };
        if let Some(worker) = self.pool.insert(command, worker) {
            worker.stop()?;
        }

        Ok(())
    }
}
