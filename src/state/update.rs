use std::sync::atomic::Ordering;

use wherehouse::package_manager::Command;

use super::{Pane, State, ToastType};

pub fn update(state: &State, event: &super::Event) -> Vec<Command> {
    match event {
        super::Event::InsertChar(ch) => {
            let mut search = state.search.lock().unwrap();
            search.query.push(*ch);
            search.list_state.select(None);
            search.query_last_changed = std::time::Instant::now();
            vec![]
        }
        super::Event::DeleteChar => {
            let mut search = state.search.lock().unwrap();
            search.query.pop();
            search.list_state.select(None);
            search.query_last_changed = std::time::Instant::now();
            vec![]
        }
        super::Event::SelectionMoved(delta) => {
            let mut search = state.search.lock().unwrap();
            if *delta < 0 {
                search.list_state.select_previous();
            } else {
                search.list_state.select_next();
            }
            let len = search.results.len();
            if let Some(sel) = search.list_state.selected() {
                let clamped = sel.clamp(0, len.saturating_sub(1));
                if clamped != sel {
                    search.list_state.select(Some(clamped));
                }
            }
            vec![]
        }
        super::Event::ContextScroll(delta) => {
            let mut scroll = state.context_scroll.lock().unwrap();
            *scroll = scroll.saturating_add_signed(*delta);
            vec![]
        }
        super::Event::PaneFocused(pane) => {
            *state.context_scroll.lock().unwrap() = 0;
            state.switch_pane(pane.clone());
            if matches!(pane, Pane::SearchResults(_)) {
                let mut search = state.search.lock().unwrap();
                if search.list_state.selected().is_none() && !search.results.is_empty() {
                    search.list_state.select(Some(0));
                }
            }
            vec![]
        }
        super::Event::Quit => {
            state.exit.store(true, Ordering::Relaxed);
            vec![]
        }
        super::Event::CommandIssued(cmd) => {
            match cmd {
                Command::PackageInfo => {
                    let backend = state.config.lock().unwrap().backend.alias().to_string();
                    let key = {
                        let search = state.search.lock().unwrap();
                        search.list_state.selected()
                            .and_then(|s| search.results.get(s))
                            .map(|r| format!("{backend}/info:{}", r.name))
                    };
                    if let Some(ref key) = key {
                        if let Some(cached) = state.package_info_cache.get(key) {
                            state.search.lock().unwrap().selected_result_info = cached;
                            return vec![];
                        }
                    }
                    state.remove_running_command(cmd);
                    state.add_toast("Loading info...".to_string(), ToastType::Progress);
                    state.add_running_command(cmd.clone());
                    vec![cmd.clone()]
                }
                Command::FilterPackages => {
                    let backend = state.config.lock().unwrap().backend.alias().to_string();
                    let query = state.search.lock().unwrap().query.clone();
                    let key = format!("{backend}/search:{query}");
                    if let Some(cached) = state.search_cache.get(&key) {
                        let mut search = state.search.lock().unwrap();
                        search.results = cached;
                        if search.updatable_only {
                            search.all_results = search.results.clone();
                        }
                        *search.list_state.offset_mut() = 0;
                        if search.list_state.selected().is_none() && !search.results.is_empty() {
                            search.list_state.select(Some(0));
                        }
                        return vec![];
                    }
                    state.remove_running_command(cmd);
                    state.add_toast("Searching...".to_string(), ToastType::Progress);
                    state.add_running_command(cmd.clone());
                    vec![cmd.clone()]
                }
                _ => {
                    let progress_msg = match cmd {
                        Command::InstallPackage => "Installing...",
                        Command::UninstallPackage => "Uninstalling...",
                        Command::CheckHealth => "Checking health...",
                        Command::Config => "Loading config...",
                        Command::UpdatePackage => "Updating...",
                        Command::UpdateAll => "Updating all...",
                        _ => "Running...",
                    };
                    state.remove_running_command(cmd);
                    state.add_toast(progress_msg.to_string(), ToastType::Progress);
                    state.add_running_command(cmd.clone());
                    vec![cmd.clone()]
                }
            }
        }
        super::Event::SearchCompleted { results, warning, query } => {
            let backend = state.config.lock().unwrap().backend.alias().to_string();
            state.search_cache.set(format!("{backend}/search:{query}"), results.clone());
            let mut search = state.search.lock().unwrap();
            search.results = results.clone();
            *search.list_state.offset_mut() = 0;

            if search.updatable_only {
                search.all_results = search.results.clone();
            }

            let should_fetch_info = if search.results.is_empty() {
                search.list_state.select(None);
                false
            } else if search.list_state.selected().is_none() {
                search.list_state.select(Some(0));
                true
            } else {
                false
            };

            if search.search_active {
                state.switch_pane(Pane::SearchResults(String::new()));
            }

            drop(search);
            state.remove_running_command(&Command::FilterPackages);
            if let Some(msg) = warning {
                state.add_toast(msg.clone(), ToastType::Info);
            }

            let mut cmds = vec![Command::CheckOutdated];
            if should_fetch_info {
                let first_name = results.first().map(|r| r.name.clone());
                if let Some(ref name) = first_name {
                    let key = format!("{backend}/info:{name}");
                    if let Some(cached) = state.package_info_cache.get(&key) {
                        state.search.lock().unwrap().selected_result_info = cached;
                        return cmds;
                    }
                }
                cmds.push(Command::PackageInfo);
            }
            cmds
        }
        super::Event::OutdatedCheckCompleted { outdated } => {
            let outdated_set: std::collections::HashSet<&str> =
                outdated.iter().map(String::as_str).collect();
            let mut search = state.search.lock().unwrap();
            for result in &mut search.results {
                if result.is_installed && outdated_set.contains(result.name.as_str()) {
                    result.update_available = true;
                }
            }
            for result in &mut search.all_results {
                if result.is_installed && outdated_set.contains(result.name.as_str()) {
                    result.update_available = true;
                }
            }
            if search.updatable_only {
                search.results = search.all_results.clone();
                search.results.retain(|r| r.update_available);
            }
            state.remove_running_command(&Command::CheckOutdated);
            vec![]
        }
        super::Event::CommandOutputReceived {
            cmd: Command::Config,
            output,
        } => {
            state.config.lock().unwrap().system_config = output.clone();
            state.switch_pane(Pane::About(output.clone()));
            state.add_toast("Config loaded".to_string(), ToastType::Success);
            state.remove_running_command(&Command::Config);
            vec![]
        }
        super::Event::CommandOutputReceived {
            cmd: Command::CheckHealth,
            output,
        } => {
            *state.healthcheck_results.lock().unwrap() = output.clone();
            state.switch_pane(Pane::About(output.clone()));
            state.add_toast("Health check complete".to_string(), ToastType::Success);
            state.remove_running_command(&Command::CheckHealth);
            vec![]
        }
        super::Event::CommandOutputReceived {
            cmd: Command::PackageInfo,
            output,
        } => {
            let backend = state.config.lock().unwrap().backend.alias().to_string();
            let key = {
                let search = state.search.lock().unwrap();
                search.list_state.selected()
                    .and_then(|s| search.results.get(s))
                    .map(|r| format!("{backend}/info:{}", r.name))
            };
            if let Some(key) = key {
                state.package_info_cache.set(key, output.clone());
            }
            state.search.lock().unwrap().selected_result_info = output.clone();
            state.remove_running_command(&Command::PackageInfo);
            vec![]
        }
        super::Event::CommandOutputReceived {
            cmd: Command::InstallPackage,
            output,
        } => {
            state.switch_pane(Pane::SearchResults(output.clone()));
            state.add_toast("Install complete".to_string(), ToastType::Success);
            invalidate_caches(state);
            state.remove_running_command(&Command::InstallPackage);
            vec![Command::FilterPackages]
        }
        super::Event::CommandOutputReceived {
            cmd: Command::UninstallPackage,
            output,
        } => {
            state.switch_pane(Pane::SearchResults(output.clone()));
            state.add_toast("Uninstall complete".to_string(), ToastType::Success);
            invalidate_caches(state);
            state.remove_running_command(&Command::UninstallPackage);
            vec![Command::FilterPackages]
        }
        super::Event::CommandOutputReceived { cmd, output } => {
            let should_refresh = matches!(cmd, Command::UpdatePackage | Command::UpdateAll);
            match cmd {
                Command::UpdatePackage | Command::Clean => {
                    state.add_toast("Update complete".to_string(), ToastType::Success);
                    invalidate_caches(state);
                }
                Command::UpdateAll => {
                    state.add_toast("All packages updated".to_string(), ToastType::Success);
                    invalidate_caches(state);
                }
                _ => {}
            }
            state.switch_pane(Pane::SearchResults(output.clone()));
            state.remove_running_command(cmd);
            if should_refresh {
                vec![Command::FilterPackages]
            } else {
                vec![]
            }
        }
        super::Event::ToggleUpdatableFilter => {
            let mut search = state.search.lock().unwrap();
            search.updatable_only = !search.updatable_only;
            if search.updatable_only {
                search.all_results = search.results.clone();
                search.results.retain(|r| r.update_available);
                let len = search.results.len();
                if let Some(sel) = search.list_state.selected() {
                    if sel >= len {
                        search.list_state.select(len.checked_sub(1));
                    }
                }
            } else if !search.all_results.is_empty() {
                search.results = search.all_results.clone();
                search.all_results.clear();
            }
            vec![]
        }
        super::Event::CommandFailed { cmd, error } => {
            if cmd == &Command::CheckOutdated {
                state.remove_running_command(cmd);
                return vec![];
            }
            if !error.contains("superseded") {
                state.add_toast(format!("{} failed: {}", cmd_name(cmd), error), ToastType::Error);
                state.remove_running_command(cmd);
            }
            vec![]
        }
        super::Event::SwitchBackend(delta) => {
            let backend = state.cycle_backend(*delta);
            state.config.lock().unwrap().backend = backend;
            state.config.lock().unwrap().system_config.clear();
            *state.healthcheck_results.lock().unwrap() = String::new();
            *state.context_scroll.lock().unwrap() = 0;
            {
                let mut search = state.search.lock().unwrap();
                search.query.clear();
                search.results.clear();
                search.all_results.clear();
                search.selected_result_info.clear();
                search.list_state.select(None);
            }
            invalidate_caches(state);
            state.package_info_cache.invalidate_all();
            state.search_cache.invalidate_all();
            state.switch_pane(Pane::About(String::new()));
            vec![Command::Config, Command::FilterPackages]
        }
        super::Event::ShowToast { message, toast_type } => {
            state.add_toast(message.clone(), *toast_type);
            vec![]
        }
    }
}

fn invalidate_caches(state: &State) {
    let backend = state.config.lock().unwrap().backend.alias().to_string();
    state.package_info_cache.invalidate_prefix(&backend);
    state.search_cache.invalidate_prefix(&backend);
}

fn cmd_name(cmd: &Command) -> &'static str {
    match cmd {
        Command::InstallPackage => "Install",
        Command::UninstallPackage => "Uninstall",
        Command::CheckHealth => "Health check",
        Command::Config => "Config",
        Command::FilterPackages => "Search",
        Command::PackageInfo => "Package info",
        Command::UpdatePackage => "Update",
        Command::UpdateAll => "Update all",
        Command::Clean => "Clean",
        Command::GeneralInfo => "Info",
        Command::CheckOutdated => "Check outdated",
    }
}
