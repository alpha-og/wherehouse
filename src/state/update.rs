use std::sync::atomic::Ordering;

use wherehouse::package_manager::Command;

use super::{Pane, State};

pub fn update(state: &State, event: &super::Event) -> Option<Command> {
    match event {
        super::Event::InsertChar(ch) => {
            let mut search = state.search.lock().unwrap();
            search.query.push(*ch);
            search.list_state.select(None);
            search.query_last_changed = std::time::Instant::now();
            None
        }
        super::Event::DeleteChar => {
            let mut search = state.search.lock().unwrap();
            search.query.pop();
            search.list_state.select(None);
            if search.query.is_empty() {
                search.results.clear();
            }
            search.query_last_changed = std::time::Instant::now();
            None
        }
        super::Event::SelectionMoved(delta) => {
            let mut search = state.search.lock().unwrap();
            if *delta < 0 {
                search.list_state.select_previous();
            } else {
                search.list_state.select_next();
            }
            None
        }
        super::Event::InputModeChanged(mode) => {
            *state.input_mode.lock().unwrap() = *mode;
            None
        }
        super::Event::PaneFocused(pane) => {
            *state.current_pane.lock().unwrap() = pane.clone();
            None
        }
        super::Event::Quit => {
            state.exit.store(true, Ordering::Relaxed);
            None
        }
        super::Event::CommandIssued(cmd) => Some(cmd.clone()),
        super::Event::SearchCompleted { results, warning: _ } => {
            let mut search = state.search.lock().unwrap();
            search.results = results.clone();
            *search.list_state.offset_mut() = 0;
            if search.results.is_empty() {
                search.list_state.select(None);
            } else if search.list_state.selected().is_none() {
                search.list_state.select(Some(0));
            }
            None
        }
        super::Event::CommandOutputReceived {
            cmd: Command::Config,
            output,
        } => {
            state.config.lock().unwrap().system_config = output.clone();
            *state.current_pane.lock().unwrap() = Pane::About(output.clone());
            None
        }
        super::Event::CommandOutputReceived {
            cmd: Command::CheckHealth,
            output,
        } => {
            *state.healthcheck_results.lock().unwrap() = output.clone();
            *state.current_pane.lock().unwrap() = Pane::About(output.clone());
            None
        }
        super::Event::CommandOutputReceived {
            cmd: Command::PackageInfo,
            output,
        } => {
            let mut search = state.search.lock().unwrap();
            search.selected_result_info = output.clone();
            drop(search);
            *state.current_pane.lock().unwrap() = Pane::SearchResults(output.clone());
            None
        }
        super::Event::CommandOutputReceived { output, .. } => {
            *state.current_pane.lock().unwrap() = Pane::SearchResults(output.clone());
            None
        }
        super::Event::CommandFailed { error, .. } => {
            *state.error.lock().unwrap() = Some(error.clone());
            None
        }
    }
}
