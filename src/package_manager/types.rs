pub type SpawnCommandResult = Result<std::process::Child, std::io::Error>;
pub type CommandResult = std::io::Result<std::process::Output>;

pub struct SpawnedCommandOutput {
    pub out: Option<String>,
    pub err: Option<String>,
}

#[derive(Clone)]
pub struct SearchResult {
    pub name: String,
    pub is_installed: bool,
    pub update_available: bool,
}
