use wherehouse::package_manager::{Command, SearchResult};

use super::{Pane, ToastType};

pub enum Event {
    InsertChar(char),
    DeleteChar,
    SelectionMoved(isize),
    PaneFocused(Pane),
    Quit,
    CommandIssued(Command),
    SearchCompleted {
        results: Vec<SearchResult>,
        warning: Option<String>,
        query: String,
    },
    CommandOutputReceived { cmd: Command, output: String },
    CommandFailed { cmd: Command, error: String },
    ShowToast { message: String, toast_type: ToastType },
    ToggleUpdatableFilter,
    ContextScroll(isize),
    OutdatedCheckCompleted { outdated: Vec<String> },
}
