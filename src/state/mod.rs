use std::time::{Duration, Instant};
use std::{
    fmt::Display,
    sync::atomic::AtomicBool,
    sync::{Arc, Mutex, MutexGuard},
};

use ratatui::widgets::ListState;
use wherehouse::package_manager::{Backend, Command, SearchResult};

pub mod event;
pub mod update;

pub use event::Event;

const TOAST_DURATION: std::time::Duration = std::time::Duration::from_secs(3);

#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Info,
    Progress,
}

impl Display for ToastType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToastType::Success => write!(f, "✓"),
            ToastType::Error => write!(f, "✗"),
            ToastType::Info => write!(f, "i"),
            ToastType::Progress => write!(f, ""),
        }
    }
}

pub struct State {
    pub exit: AtomicBool,
    pub config: Arc<Mutex<Config>>,
    pub about: Arc<Mutex<String>>,

    pub current_pane: Arc<Mutex<Pane>>,
    pub search: Arc<Mutex<SearchState>>,
    pub healthcheck_results: Arc<Mutex<String>>,

    pub toasts: Arc<Mutex<Vec<Toast>>>,
    pub running_commands: Arc<Mutex<Vec<Command>>>,
    pub context_scroll: Arc<Mutex<usize>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            exit: AtomicBool::new(false),
            config: Arc::new(Mutex::new(Config::default())),
            about: Arc::new(Mutex::new(String::default())),

            current_pane: Arc::new(Mutex::new(Pane::About(String::default()))),
            search: Arc::new(Mutex::new(SearchState::default())),
            healthcheck_results: Arc::new(Mutex::new(String::default())),

            toasts: Arc::new(Mutex::new(Vec::new())),
            running_commands: Arc::new(Mutex::new(Vec::new())),
            context_scroll: Arc::new(Mutex::new(0)),
        }
    }

    pub fn exit(&self) -> bool {
        self.exit.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn set_exit(&self, exit: bool) {
        self.exit.store(exit, std::sync::atomic::Ordering::Relaxed);
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

    pub fn search(&self) -> MutexGuard<'_, SearchState> {
        self.search.lock().unwrap()
    }

    pub fn add_toast(&self, message: String, toast_type: ToastType) {
        self.toasts.lock().unwrap().push(Toast {
            message,
            toast_type,
            created_at: Instant::now(),
        });
    }

    pub fn clean_expired_toasts(&self) {
        self.toasts
            .lock()
            .unwrap()
            .retain(|t| t.toast_type == ToastType::Progress || t.created_at.elapsed() < TOAST_DURATION);
    }

    pub fn add_running_command(&self, cmd: Command) {
        self.running_commands.lock().unwrap().push(cmd);
    }

    pub fn remove_running_command(&self, cmd: &Command) {
        self.running_commands.lock().unwrap().retain(|c| c != cmd);
    }

    pub fn is_any_command_running(&self) -> bool {
        !self.running_commands.lock().unwrap().is_empty()
    }

    pub fn debounce_search(&self) -> bool {
        let mut search = self.search.lock().unwrap();
        if search.query_last_changed > search.query_last_searched
            && search.query_last_changed.elapsed() >= Duration::from_millis(300)
        {
            search.query_last_searched = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn current_toast(&self) -> Option<Toast> {
        if self.is_any_command_running() {
            return self.toasts.lock().unwrap().iter().rfind(|t| t.toast_type == ToastType::Progress).cloned();
        }
        self.toasts.lock().unwrap().iter().rfind(|t| t.toast_type != ToastType::Progress).cloned()
    }
}

#[derive(Clone)]
pub enum Pane {
    SearchResults(String),
    About(String),
    Context,
}

pub type SearchResults = Vec<SearchResult>;

#[derive(Clone)]
pub struct SearchState {
    pub query: String,
    pub results: SearchResults,
    pub selected_result: usize,
    pub selected_result_info: String,
    pub list_state: ListState,
    pub query_last_changed: Instant,
    pub query_last_searched: Instant,
    pub search_active: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::default(),
            results: SearchResults::default(),
            selected_result: usize::default(),
            selected_result_info: String::default(),
            list_state: ListState::default(),
            query_last_changed: Instant::now(),
            query_last_searched: Instant::now(),
            search_active: false,
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
