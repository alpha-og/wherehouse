use wherehouse::package_manager::{Command, PackageLocality};

use super::Pane;

pub enum Event {
    InsertChar(char),
    DeleteChar,
    SearchSourceChanged(PackageLocality),
    SelectionMoved(isize),
    InputModeChanged(super::InputMode),
    PaneFocused(Pane),
    Quit,
    CommandIssued(Command),
    SearchCompleted(Vec<String>),
    CommandOutputReceived { cmd: Command, output: String },
    CommandFailed { cmd: Command, error: String },
}
