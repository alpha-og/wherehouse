use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy)]
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

impl Display for InputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Insert => write!(f, "INSERT"),
        }
    }
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Homebrew => write!(f, "Homebrew"),
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

pub enum SearchSource {
    Remote,
    Local,
}

pub type SearchResults = Vec<SearchResult>;

pub struct SearchState {
    pub query: Arc<Mutex<String>>,
    pub results: Arc<Mutex<SearchResults>>,
    pub selected_result: Arc<Mutex<usize>>,
    pub source: Arc<Mutex<SearchSource>>,
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
    pub search: SearchState,
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
            query: Arc::new(Mutex::new(String::default())),
            results: Arc::new(Mutex::new(SearchResults::default())),
            selected_result: Arc::new(Mutex::new(usize::default())),
            source: Arc::new(Mutex::new(SearchSource::Local)),
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            current_pane: Arc::new(Mutex::new(Pane::SearchInput)),
            input_mode: Arc::new(Mutex::new(InputMode::Insert)),
            search: SearchState::default(),
            should_quit: Arc::new(Mutex::new(false)),
            config: Arc::new(Mutex::new(Config::default())),
        }
    }
}
