use std::{process::Command, sync::mpsc::Sender};

use crate::state::{self, SearchResult, State, StateEvent, StateResponse};

pub struct Worker {
    tx_state: Sender<StateEvent>,
    search_query: String,
}

impl Worker {
    pub fn new(tx_state: Sender<StateEvent>) -> Self {
        Self {
            tx_state,
            search_query: String::default(),
        }
    }
    pub fn run(&self) {
        loop {
            if State::sync_worker(&self.tx_state) {
                if let Some(StateResponse::SearchQuery(search_query)) =
                    State::get(&self.tx_state, state::StateItemType::SearchQuery)
                {
                    if search_query != self.search_query {
                        self.brew_search(search_query);
                    }
                }
                if let Some(StateResponse::ShouldQuit(should_quit)) =
                    State::get(&self.tx_state, state::StateItemType::ShouldQuit)
                {
                    if should_quit {
                        break;
                    }
                }
            };
        }
    }

    fn brew_search(&self, search_query: String) {
        if !search_query.is_empty() {
            let out = String::from_utf8(
                Command::new("brew")
                    .arg("search")
                    .arg(&search_query)
                    .output()
                    .expect("ls failed to execute")
                    .stdout,
            )
            .expect("unable to parse string");
            let search_results = out
                .split("\n")
                .filter_map(|result| {
                    if !result.is_empty() {
                        Some(SearchResult {
                            display_text: result.to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if search_results.len() > 0 {
                State::set(
                    &self.tx_state,
                    StateEvent::SetSearchResults(Some(search_results)),
                )
                .unwrap();
            } else {
                State::set(&self.tx_state, StateEvent::SetSearchResults(None)).unwrap();
            }
        } else {
            State::set(&self.tx_state, StateEvent::SetSearchResults(None)).unwrap();
        }
        State::set(&self.tx_state, StateEvent::SetSearchSelectedResult(0)).unwrap();
    }
}
