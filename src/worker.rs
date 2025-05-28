use std::{process::Command, sync::mpsc};

use crate::app::{self, TuiEvent};

pub enum WorkerEvent {
    Search(String),
}

// pub struct Worker {
//     is_busy
// }

pub fn worker_thread(rx: mpsc::Receiver<WorkerEvent>, tx_tui: mpsc::Sender<app::TuiEvent>) {
    loop {
        match rx.recv().unwrap() {
            WorkerEvent::Search(query) => {
                brew_search(query, tx_tui.clone());
            }
        }
    }
}

fn brew_search(search_query: String, tx: mpsc::Sender<TuiEvent>) {
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
            .map(|result| result.to_string())
            .collect::<Vec<_>>();
        let search_results = TuiEvent::SearchResult(app::SearchResults {
            results: search_results,
            selected_index: 0,
        });
        tx.send(search_results).unwrap();
    } else {
        let search_results = TuiEvent::SearchResult(app::SearchResults {
            results: Vec::new(),
            selected_index: 0,
        });

        tx.send(search_results).unwrap();
    }
}
