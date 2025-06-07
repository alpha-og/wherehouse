use std::{
    fmt::Display,
    sync::{Arc, Mutex, MutexGuard},
};

use ratatui::widgets::ListState;
use wherehouse::package_manager::{Backend, PackageLocality};

pub struct State {
    exit: Arc<Mutex<bool>>,
    pub config: Arc<Mutex<Config>>,
    about: Arc<Mutex<String>>,

    current_pane: Arc<Mutex<Pane>>,
    input_mode: Arc<Mutex<InputMode>>,
    pub search: Arc<Mutex<SearchState>>,
    healthcheck_results: Arc<Mutex<String>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            exit: Arc::new(Mutex::new(false)),
            config: Arc::new(Mutex::new(Config::default())),
            about: Arc::new(Mutex::new(String::default())),

            current_pane: Arc::new(Mutex::new(Pane::SearchInput)),
            input_mode: Arc::new(Mutex::new(InputMode::Insert)),
            search: Arc::new(Mutex::new(SearchState::default())),
            healthcheck_results: Arc::new(Mutex::new(String::default())),
        }
    }

    pub fn exit(&self) -> bool {
        match self.exit.try_lock() {
            Ok(exit) => *exit,
            Err(_) => false,
        }
    }
    pub fn set_exit(&self, exit: bool) {
        *self.exit.lock().unwrap() = exit;
    }

    pub fn config(&self) -> MutexGuard<'_, Config> {
        self.config.lock().unwrap()
    }

    pub fn about(&self) -> String {
        (*self.about.lock().unwrap()).clone()
    }
    pub fn set_about(&self, content: String) {
        *self.about.lock().unwrap() = content;
    }

    pub fn current_pane(&self) -> Pane {
        (*self.current_pane.lock().unwrap()).clone()
    }
    pub fn set_current_pane(&self, pane: Pane) {
        *self.current_pane.lock().unwrap() = pane;
    }

    pub fn input_mode(&self) -> InputMode {
        *self.input_mode.lock().unwrap()
    }
    pub fn set_input_mode(&self, input_mode: InputMode) {
        *self.input_mode.lock().unwrap() = input_mode;
    }

    pub fn search(&self) -> MutexGuard<'_, SearchState> {
        self.search.lock().unwrap()
    }

    pub fn healthcheck_results(&self) -> String {
        (*self.healthcheck_results.lock().unwrap()).clone()
    }
    pub fn set_healthcheck_results(&self, result: String) {
        *self.healthcheck_results.lock().unwrap() = result;
    }
}

#[derive(Clone)]
pub enum Pane {
    SearchInput,
    SearchResults(String),
    About(String),
    Context,
}

#[derive(Clone, Copy)]
pub enum InputMode {
    Normal,
    Insert,
}

impl Display for InputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Insert => write!(f, "INSERT"),
        }
    }
}

pub type SearchResults = Vec<String>;

#[derive(Clone)]
pub struct SearchState {
    pub query: String,
    pub results: SearchResults,
    pub selected_result: usize,
    pub selected_result_info: String,
    pub list_state: ListState,
    pub source: PackageLocality,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::default(),
            results: SearchResults::default(),
            selected_result: usize::default(),
            selected_result_info: String::default(),
            list_state: ListState::default(),
            source: PackageLocality::Local,
        }
    }
}

pub struct Config {
    pub backend: Backend,
    pub system_config: String,
    pub app_version: String,
    pub app_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backend: Backend::default(),
            app_version: String::from("0.1.0"),
            app_name: String::from("WhereHouse"),
            system_config: String::default(),
        }
    }
}
