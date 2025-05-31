use std::{
    process::Command,
    sync::{Arc, mpsc::Receiver},
    thread,
    time::Duration,
};

use crate::state::{SearchResult, State};

pub enum WorkerEvent {
    UpdateSearch,
}

pub struct Worker {
    rx: Receiver<WorkerEvent>,
    state: Arc<State>,
    search_query: String,
}

impl Worker {
    pub fn new(rx: Receiver<WorkerEvent>, state: Arc<State>) -> Self {
        Self {
            rx,
            state,
            search_query: String::default(),
        }
    }
    pub fn run(&mut self) {
        loop {
            if let Ok(event) = self.rx.try_recv() {
                match event {
                    WorkerEvent::UpdateSearch => {
                        if let Ok(search_query) = self.state.search.query.try_lock() {
                            if *search_query != self.search_query {
                                self.search_query = search_query.clone();
                                drop(search_query);
                                self.brew_search();
                            }
                        }
                    }
                }
            }
            if let Ok(should_quit) = self.state.should_quit.try_lock() {
                if *should_quit {
                    break;
                }
            }
        }
    }

    fn brew_search(&self) {
        if !self.search_query.is_empty() {
            let out = String::from_utf8(
                Command::new("brew")
                    .arg("search")
                    .arg(&self.search_query)
                    .output()
                    .expect("ls failed to execute")
                    .stdout,
            )
            .expect("unable to parse string");
            let results = out
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
            if let Ok(search_query) = self.state.search.query.lock() {
                if *search_query != self.search_query {
                    return;
                }
            }
            if let Ok(mut search_results) = self.state.search.results.lock() {
                if results.len() > 0 {
                    *search_results = results;
                } else {
                    *search_results = Vec::new();
                }
            }
        } else {
            if let Ok(mut search_results) = self.state.search.results.lock() {
                *search_results = Vec::new();
            }
        }
        if let Ok(mut selected_search_result) = self.state.search.selected_result.lock() {
            *selected_search_result = 0;
        }
    }
}
