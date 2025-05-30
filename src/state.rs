use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, SendError, Sender},
};

#[derive(Clone)]
pub enum InputMode {
    Normal,
    Insert,
}

pub enum PackageManager {
    Homebrew,
}

#[derive(Clone)]
pub enum Pane {
    SearchInput,
    SearchResults,
}

pub enum StateResponse {
    SearchQuery(SearchQuery),
    SearchResults(Option<Vec<SearchResult>>),
    SelectedSearchResult(usize),
    CurrentPane(Pane),
    InputMode(InputMode),
    ShouldQuit(bool),
    SyncWorker(bool), // true if not synced ,else false
    SyncTui(bool),    // true if not synced ,else false
}

pub enum StateItemType {
    SearchQuery,
    SearchResults,
    SearchSelectedResult,
    ShouldQuit,
    InputMode,
    CurrentPane,
}

pub enum StateEvent {
    SetSearchQuery(String),
    GetSearchQuery(Sender<StateEvent>),
    GetSearchResults(Sender<StateEvent>),
    SetSearchResults(Option<Vec<SearchResult>>),
    SetSearchSelectedResult(usize),
    GetSearchSelectedResult(Sender<StateEvent>),
    SetCurrentPane(Pane),
    GetCurrentPane(Sender<StateEvent>),
    SetInputMode(InputMode),
    GetInputMode(Sender<StateEvent>),
    SetShouldQuit(bool),
    GetShouldQuit(Sender<StateEvent>),
    SyncWorker(Sender<StateEvent>),
    SyncTui(Sender<StateEvent>),
    Response(StateResponse),
}

impl std::fmt::Display for InputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Insert => write!(f, "INSERT"),
        }
    }
}

#[derive(Default, Clone)]
pub struct SearchResult {
    pub display_text: String,
}

type SearchQuery = String;

#[derive(Default)]
struct SearchState {
    query: SearchQuery,
    results: Option<Vec<SearchResult>>,
    selected_result: usize,
}

pub struct State {
    current_pane: Pane,
    input_mode: InputMode,
    search: SearchState,
    package_manager: PackageManager,
    should_quit: bool,
    synced_worker: bool,
    synced_tui: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            current_pane: Pane::SearchInput,
            input_mode: InputMode::Insert,
            search: SearchState::default(),
            package_manager: PackageManager::Homebrew,
            should_quit: false,
            synced_worker: false,
            synced_tui: false,
        }
    }
}

impl State {
    pub fn manage(&mut self, rx: Receiver<StateEvent>) {
        loop {
            if let Ok(event) = rx.recv() {
                match event {
                    StateEvent::SetSearchQuery(query) => {
                        self.search.query = query;
                        self.reset_sync_status();
                    }
                    StateEvent::SetSearchResults(search_results) => {
                        self.search.results = search_results;
                        self.reset_sync_status();
                    }
                    StateEvent::GetSearchQuery(tx) => tx
                        .send(StateEvent::Response(StateResponse::SearchQuery(
                            self.search.query.clone(),
                        )))
                        .unwrap(),
                    StateEvent::GetSearchResults(tx) => {
                        let search_results = match &self.search.results {
                            Some(results) => {
                                Some(results.iter().map(|result| result.clone()).collect())
                            }
                            None => None,
                        };
                        tx.send(StateEvent::Response(StateResponse::SearchResults(
                            search_results,
                        )))
                        .unwrap()
                    }
                    StateEvent::SetCurrentPane(pane) => {
                        self.current_pane = pane;
                        self.reset_sync_status();
                    }
                    StateEvent::GetCurrentPane(tx) => tx
                        .send(StateEvent::Response(StateResponse::CurrentPane(
                            self.current_pane.clone(),
                        )))
                        .unwrap(),
                    StateEvent::SetInputMode(input_mode) => {
                        self.input_mode = input_mode;
                        self.reset_sync_status();
                    }
                    StateEvent::GetInputMode(tx) => tx
                        .send(StateEvent::Response(StateResponse::InputMode(
                            self.input_mode.clone(),
                        )))
                        .unwrap(),
                    StateEvent::SetShouldQuit(should_quit) => {
                        self.should_quit = should_quit;
                        self.reset_sync_status();
                    }
                    StateEvent::GetShouldQuit(tx) => tx
                        .send(StateEvent::Response(StateResponse::ShouldQuit(
                            self.should_quit,
                        )))
                        .unwrap(),

                    StateEvent::SetSearchSelectedResult(selected_result) => {
                        self.search.selected_result = selected_result;
                        self.reset_sync_status();
                    }
                    StateEvent::GetSearchSelectedResult(tx) => tx
                        .send(StateEvent::Response(StateResponse::SelectedSearchResult(
                            self.search.selected_result,
                        )))
                        .unwrap(),
                    StateEvent::SyncWorker(tx) => {
                        tx.send(StateEvent::Response(StateResponse::SyncWorker(
                            !self.synced_worker,
                        )))
                        .unwrap();
                        self.synced_worker = true;
                    }
                    StateEvent::SyncTui(tx) => {
                        tx.send(StateEvent::Response(StateResponse::SyncTui(
                            !self.synced_tui,
                        )))
                        .unwrap();
                        self.synced_tui = true;
                    }

                    _ => {}
                }
            } else {
            };
        }
    }

    fn reset_sync_status(&mut self) {
        self.synced_worker = false;
        self.synced_tui = false;
    }

    pub fn get(
        // rx: &Receiver<StateEvent>,
        tx_state: &Sender<StateEvent>,
        state_item_type: StateItemType,
    ) -> Option<StateResponse> {
        let (tx, rx) = mpsc::channel::<StateEvent>();
        match state_item_type {
            StateItemType::SearchQuery => tx_state.send(StateEvent::GetSearchQuery(tx)),
            StateItemType::SearchResults => tx_state.send(StateEvent::GetSearchResults(tx)),
            StateItemType::SearchSelectedResult => {
                tx_state.send(StateEvent::GetSearchSelectedResult(tx))
            }
            StateItemType::CurrentPane => tx_state.send(StateEvent::GetCurrentPane(tx)),
            StateItemType::InputMode => tx_state.send(StateEvent::GetInputMode(tx)),
            StateItemType::ShouldQuit => tx_state.send(StateEvent::GetShouldQuit(tx)),
        };
        match rx.recv() {
            Ok(StateEvent::Response(response)) => Some(response),
            _ => None,
        }
    }
    pub fn set(
        tx_state: &Sender<StateEvent>,
        event: StateEvent,
    ) -> Result<(), SendError<StateEvent>> {
        tx_state.send(event)
    }

    pub fn sync_worker(tx_state: &Sender<StateEvent>) -> bool {
        let (tx, rx) = mpsc::channel::<StateEvent>();
        tx_state.send(StateEvent::SyncWorker(tx));
        if let Ok(StateEvent::Response(StateResponse::SyncWorker(sync_status))) = rx.recv() {
            sync_status
        } else {
            true
        }
    }

    pub fn sync_tui(tx_state: &Sender<StateEvent>) -> bool {
        let (tx, rx) = mpsc::channel::<StateEvent>();
        tx_state.send(StateEvent::SyncTui(tx));
        if let Ok(StateEvent::Response(StateResponse::SyncTui(sync_status))) = rx.recv() {
            sync_status
        } else {
            true
        }
    }
}
