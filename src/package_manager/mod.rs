pub mod backend;
pub mod command;
pub mod error;
pub mod homebrew;
pub mod manager;
pub mod types;
pub mod util;

pub use backend::Backend;
pub use command::Command;
pub use manager::PackageManager;
pub use types::{CommandResult, SearchResult, SpawnCommandResult, SpawnedCommandOutput};
pub use util::{command, detect_package_manager, handle_spawned_command, spawn_command};
