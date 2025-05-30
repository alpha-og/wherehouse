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

#[derive(Default)]
struct SearchState {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected_result: usize,
}

pub struct State {
    pub current_pane: Arc<Mutex<Pane>>,
    pub input_mode: Arc<Mutex<InputMode>>,
    pub search: Arc<Mutex<SearchState>>,
    pub should_quit: Arc<Mutex<bool>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            current_pane: Arc::new(Mutex::new(Pane::SearchInput)),
            input_mode: Arc::new(Mutex::new(InputMode::Insert)),
            search: Arc::new(Mutex::new(SearchState::default())),
            should_quit: Arc::new(Mutex::new(false)),
        }
    }
}
