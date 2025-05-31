use std::sync::{Arc, Mutex};

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

pub type SearchResults = Vec<SearchResult>;

#[derive(Default)]
pub struct SearchState {
    pub query: Arc<Mutex<String>>,
    pub results: Arc<Mutex<SearchResults>>,
    pub selected_result: Arc<Mutex<usize>>,
}

pub struct State {
    pub current_pane: Arc<Mutex<Pane>>,
    pub input_mode: Arc<Mutex<InputMode>>,
    pub search: SearchState,
    pub should_quit: Arc<Mutex<bool>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            current_pane: Arc::new(Mutex::new(Pane::SearchInput)),
            input_mode: Arc::new(Mutex::new(InputMode::Insert)),
            search: SearchState::default(),
            should_quit: Arc::new(Mutex::new(false)),
        }
    }
}
