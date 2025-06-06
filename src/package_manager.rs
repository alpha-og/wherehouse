use std::{
    ffi::OsStr,
    fmt::Display,
    io::{BufRead, BufReader},
    process::{Child, Output, Stdio},
    sync::mpsc::{Receiver, channel},
    thread,
};

pub mod homebrew;

pub type SpawnCommandResult = Result<std::process::Child, std::io::Error>;
pub type CommandResult = std::io::Result<std::process::Output>;

/// spawn a non-blocking command returning the Child
pub fn spawn_command<I, S>(alias: &'static str, args: I) -> SpawnCommandResult
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    std::process::Command::new(alias)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

pub struct SpawnedCommandOutput {
    pub out: Option<String>,
    pub err: Option<String>,
}

pub fn handle_spawned_command(
    rx: Receiver<bool>,
    mut child: Child,
) -> Option<SpawnedCommandOutput> {
    // handle the stdout stream in another thread
    let stdout = child.stdout.take().expect("no stdout");
    let (tx_stdout, rx_stdout) = channel::<String>();
    let stdout_handle = thread::spawn(move || {
        let mut out = String::new();
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(content) = line {
                out.push_str(&content);
                out.push('\n');
            }
        }
        tx_stdout.send(out).unwrap();
    });

    // handle the stderr stream in another thread
    let stderr = child.stderr.take().expect("no stdout");
    let (tx_stderr, rx_stderr) = channel::<String>();
    let stderr_handle = thread::spawn(move || {
        let mut err = String::new();
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(content) = line {
                err.push_str(&content);
                err.push('\n');
            }
        }
        tx_stderr.send(err).unwrap();
    });

    loop {
        // check if the spawned command has exited
        if let Ok(Some(_)) = child.try_wait() {
            // collect the stdout stream as a string via the channel
            let out = match rx_stdout.recv() {
                Ok(out) => Some(out),
                Err(_) => None,
            };
            // collect the stderr stream as a string via the channel
            let err = match rx_stderr.recv() {
                Ok(err) => Some(err),
                Err(_) => None,
            };
            return Some(SpawnedCommandOutput { out, err });
        }
        // check if spawned command is stale and terminal if it is stale
        if let Ok(is_stale) = rx.try_recv() {
            if is_stale {
                child.kill().unwrap(); // kill the spawned command

                // join the stream handling threads
                stdout_handle.join().expect("Failed to join stdout thread");
                stderr_handle.join().expect("Failed to join stderr thread");
                break;
            }
        }
    }
    None
}

/// create a blocking command and run until completion returning the output wrapped in a Result
pub fn command<I, S>(alias: &'static str, args: I) -> CommandResult
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    std::process::Command::new(alias).args(args).output()
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

#[derive(PartialEq, Eq, Hash)]
pub enum Command {
    FilterPackages,
    Config,
    PackageInfo,
    GeneralInfo,
    CheckHealth,
    InstallPackage,
    UninstallPackage,
    UpdatePackage,
    Clean,
}

pub trait PackageManager: Send + Sync + 'static {
    fn alias(&self) -> &'static str;
    fn filter_packages(
        &self,
        rx: Receiver<bool>,
        source: PackageLocality,
        pattern: String,
    ) -> Result<Vec<String>, String>;
    fn package_manager_config(&self, rx: Receiver<bool>) -> Result<String, String>;
    fn package_info(&self, rx: Receiver<bool>, package_name: String) -> Result<String, String>;
    fn check_health(&self, rx: Receiver<bool>) -> Result<String, String>;
    fn clean(&self, rx: Receiver<bool>) -> Result<String, String>;
    fn install_package(&self, rx: Receiver<bool>, package_name: String) -> Result<(), String>;
    fn update_package(&self, rx: Receiver<bool>, package_name: String) -> Result<(), String>;
    fn uninstall_package(&self, rx: Receiver<bool>, package_name: String) -> Result<(), String>;
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
