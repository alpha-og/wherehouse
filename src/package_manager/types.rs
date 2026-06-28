use std::fmt::Display;

pub type SpawnCommandResult = Result<std::process::Child, std::io::Error>;
pub type CommandResult = std::io::Result<std::process::Output>;

pub struct SpawnedCommandOutput {
    pub out: Option<String>,
    pub err: Option<String>,
}

#[derive(Clone, Copy)]
pub enum PackageLocality {
    Local,
    Remote,
}

impl Display for PackageLocality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "LOCAL"),
            Self::Remote => write!(f, "REMOTE"),
        }
    }
}
