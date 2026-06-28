use std::{
    ffi::OsStr,
    io::{BufRead, BufReader},
    process::{Child, Stdio},
    sync::{Arc, mpsc::{Receiver, channel}},
    thread,
};

use super::backend::Backend;
use super::manager::PackageManager;
use super::types::{CommandResult, SpawnCommandResult, SpawnedCommandOutput};

pub fn spawn_command<I, S>(backend: Backend, args: I) -> SpawnCommandResult
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    std::process::Command::new(backend.alias())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

pub fn handle_spawned_command(
    rx: Receiver<bool>,
    mut child: Child,
) -> Option<SpawnedCommandOutput> {
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
        if let Ok(Some(_)) = child.try_wait() {
            let out = match rx_stdout.recv() {
                Ok(out) => Some(out),
                Err(_) => None,
            };
            let err = match rx_stderr.recv() {
                Ok(err) => Some(err),
                Err(_) => None,
            };
            return Some(SpawnedCommandOutput { out, err });
        }
        if let Ok(is_stale) = rx.try_recv() {
            if is_stale {
                child.kill().unwrap();
                stdout_handle.join().expect("Failed to join stdout thread");
                stderr_handle.join().expect("Failed to join stderr thread");
                break;
            }
        }
    }
    None
}

pub fn command<I, S>(backend: Backend, args: I) -> CommandResult
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    std::process::Command::new(backend.alias())
        .args(args)
        .output()
}

pub fn detect_package_manager() -> Arc<dyn PackageManager> {
    let backend = Backend::default();
    Backend::package_manager_from_backend(backend)
}
