use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, mpsc};

use tracing::info;
use wherehouse::package_manager::{Command, PackageManager};

use crate::state::Event;
use crate::state::State;
use super::worker::Worker;

pub struct TaskManager<T> {
    state: Arc<State>,
    package_manager: Arc<T>,
    tx: Sender<Event>,
    pool: HashMap<Command, Worker>,
}

impl<T: PackageManager + Send + Sync + 'static> TaskManager<T> {
    pub fn new(state: Arc<State>, package_manager: Arc<T>, tx: Sender<Event>) -> Self {
        Self {
            state,
            package_manager,
            tx,
            pool: HashMap::default(),
        }
    }

    pub fn execute(&mut self, command: Command) -> color_eyre::Result<()> {
        let state = self.state.clone();
        let package_manager = self.package_manager.clone();
        let tx = self.tx.clone();
        let (tx_task, rx_task) = mpsc::channel::<bool>();

        let worker = match command {
            Command::FilterPackages => Worker::new(tx_task, move || {
                let search = state.search();
                let query = search.query.clone();
                drop(search);
                info!("Command::FilterPackages => {query}");
                let result = package_manager.filter_packages(rx_task, query.clone());
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
                let result = package_manager.package_info(rx_task, package_name);
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
                let result = package_manager.install_package(rx_task, package_name);
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
                let result = package_manager.uninstall_package(rx_task, package_name);
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
                let result = package_manager.check_health(rx_task);
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
                let result = package_manager.update_package(rx_task, package_name);
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
                let result = package_manager.update_all_packages(rx_task);
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
                let result = package_manager.check_outdated(rx_task);
                let outdated = match result {
                    Ok(names) => names,
                    Err(_) => Vec::new(),
                };
                let _ = tx.send(Event::OutdatedCheckCompleted { outdated });
            }),
            Command::Config => Worker::new(tx_task, move || {
                let result = package_manager.package_manager_config(rx_task);
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
