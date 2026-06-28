use std::fmt;

#[derive(Debug)]
pub enum PackageManagerError {
    /// The package manager binary was not found on PATH.
    /// The String is the alias that was looked up (e.g. "brew").
    NotInstalled(String),

    /// The command executed but exited with a non-zero status.
    /// The String contains stderr or a description of what went wrong.
    ExecutionFailed(String),

    /// A package could not be found in any configured repository.
    /// The String is the package name that was searched for.
    PackageNotFound(String),

    /// An OS-level I/O error occurred (permission denied, broken pipe, etc.).
    IoError(std::io::Error),

    /// The package manager does not support the requested operation.
    /// The String describes what was attempted.
    UnsupportedOperation(String),

    /// The command was killed because a newer operation replaced it.
    StaleCommand,
}

impl fmt::Display for PackageManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInstalled(name) => write!(f, "{name} is not installed on this system"),
            Self::ExecutionFailed(msg) => write!(f, "command failed: {msg}"),
            Self::PackageNotFound(name) => write!(f, "package not found: {name}"),
            Self::IoError(e) => write!(f, "I/O error: {e}"),
            Self::UnsupportedOperation(op) => write!(f, "unsupported operation: {op}"),
            Self::StaleCommand => write!(f, "command was superseded and killed"),
        }
    }
}

impl std::error::Error for PackageManagerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PackageManagerError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}
