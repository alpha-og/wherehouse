use std::{
    fmt::Display,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::commands::PackageManager;

#[derive(Clone, Copy)]
pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Clone)]
pub enum Pane {
    SearchInput,
    SearchResults,
    Info,
}

impl Display for InputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Insert => write!(f, "INSERT"),
        }
    }
}

impl Display for SearchSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "Local"),
            Self::Remote => write!(f, "Remote"),
        }
    }
}

#[derive(Default, Clone)]
pub struct SearchResult {
    pub display_text: String,
}

#[derive(Copy, Clone)]
pub enum SearchSource {
    Remote,
    Local,
}

pub type SearchResults = Vec<SearchResult>;

pub struct SearchState {
    pub query: String,
    pub results: SearchResults,
    pub selected_result: usize,
    pub source: SearchSource,
}

pub struct Config {
    pub package_manager: PackageManager,
    pub package_manager_version: String,
    pub app_version: String,
    pub app_name: String,
}

pub struct State {
    pub current_pane: Arc<Mutex<Pane>>,
    pub input_mode: Arc<Mutex<InputMode>>,
    pub search: Arc<Mutex<SearchState>>,
    pub should_quit: Arc<Mutex<bool>>,
    pub config: Arc<Mutex<Config>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            package_manager: PackageManager::Homebrew,
            package_manager_version: String::default(),
            app_version: String::default(),
            app_name: String::from("WhereHouse"),
        }
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::default(),
            results: SearchResults::default(),
            selected_result: usize::default(),
            source: SearchSource::Local,
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            current_pane: Arc::new(Mutex::new(Pane::SearchInput)),
            input_mode: Arc::new(Mutex::new(InputMode::Insert)),
            search: Arc::new(Mutex::new(SearchState::default())),
            should_quit: Arc::new(Mutex::new(false)),
            config: Arc::new(Mutex::new(Config::default())),
        }
    }
    pub fn current_pane(&self) -> MutexGuard<'_, Pane> {
        self.current_pane.lock().unwrap()
    }
}
