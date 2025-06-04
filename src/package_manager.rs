use std::{
    ffi::OsStr,
    process::{Command, Stdio},
};

mod homebrew;

pub type SpawnCommandResult = Result<std::process::Child, std::io::Error>;
pub type CommandResult = std::io::Result<std::process::Output>;

/// spawn a non-blocking command returning the Child
pub fn spawn_command<I, S>(alias: &'static str, args: I) -> SpawnCommandResult
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(alias)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

/// create a blocking command and run until completion returning the output wrapped in a Result
pub fn command<I, S>(alias: &'static str, args: I) -> CommandResult
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(alias).args(args).output()
}

pub enum PackageLocality {
    Local,
    Remote,
}

trait PackageManager {
    fn filter_packages(source: PackageLocality, pattern: String) -> Result<Vec<String>, String>;
    fn check_health() -> Result<String, String>;
    fn clean() -> Result<String, String>;
    fn package_info(package_name: String) -> Result<String, String>;
    fn install(package_name: String) -> Result<(), String>;
    fn upgrade(package_name: String) -> Result<(), String>;
    fn uninstall(package_name: String) -> Result<(), String>;
}

// enum PackageManager {
//     Homebrew,
//     Pacman,
//     Apt,
//     Winget,
// }
//
// type Alias = &'static str;
//
// impl From<PackageManager> for Alias {
//     fn from(value: PackageManager) -> Self {
//         match value {
//             PackageManager::Homebrew => "brew",
//             PackageManager::Pacman => "pacman",
//             PackageManager::Apt => "apt",
//             PackageManager::Winget => "winget",
//         }
//     }
// }
//
//
