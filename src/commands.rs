use std::{
    ffi::OsStr,
    fmt::Display,
    io::{BufRead, BufReader, Read},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        mpsc::{Receiver, channel},
    },
    thread,
};

use tracing::info;

use crate::{
    state::{SearchResults, SearchSource},
    task_manager::TaskEvent,
};

#[derive(PartialEq, Eq, Hash)]
pub enum CommandType {
    Search,
    Config,
    Info,
    GeneralInfo,
    Healthcheck,
    Install,
    Uninstall,
    Update,
}

pub fn search(
    rx_task: Receiver<TaskEvent>,
    query: String,
    source: SearchSource,
) -> Option<SearchResults> {
    if !query.is_empty() {
        let mut child = match source {
            SearchSource::Remote => {
                match Command::new("brew")
                    .arg("search")
                    .arg(query)
                    .stdout(Stdio::piped())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(_) => return None,
                }
            }
            SearchSource::Local => match Command::new("brew")
                .arg("ls")
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(_) => return None,
            },
        };

        let stdout = child.stdout.take().expect("no stdout");
        let (tx_output, rx_output) = channel::<String>();
        let stdout_handle = thread::spawn(move || {
            let mut output = String::new();
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(content) = line {
                    output.push_str(&content);
                    output.push('\n');
                }
            }
            tx_output.send(output).unwrap();
        });
        loop {
            if let Ok(Some(_)) = child.try_wait() {
                let results = match rx_output.recv() {
                    Ok(output) => output
                        .split("\n")
                        .filter_map(|result| {
                            if !result.is_empty() {
                                Some(result.to_string())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>(),

                    _ => Vec::default(),
                };
                return Some(results);
            }
            if let Ok(task_event) = rx_task.try_recv() {
                match task_event {
                    TaskEvent::Stop => {
                        child.kill().unwrap();
                        stdout_handle.join().expect("Failed to join stdout thread");
                        break;
                    }
                }
            }
        }
    }
    None
}

pub fn info(rx_task: Receiver<TaskEvent>, package_name: String) -> Option<String> {
    if !package_name.is_empty() {
        let mut child = match Command::new("brew")
            .arg("info")
            .arg(package_name)
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return None,
        };

        let stdout = child.stdout.take().expect("no stdout");
        let (tx_output, rx_output) = channel::<String>();
        let stdout_handle = thread::spawn(move || {
            let mut output = String::new();
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(content) = line {
                    output.push_str(&content);
                    output.push('\n');
                }
            }
            tx_output.send(output).unwrap();
        });
        loop {
            if let Ok(Some(_)) = child.try_wait() {
                let results = match rx_output.recv() {
                    Ok(output) => output,
                    _ => String::default(),
                };
                return Some(results);
            }
            if let Ok(task_event) = rx_task.try_recv() {
                match task_event {
                    TaskEvent::Stop => {
                        child.kill().unwrap();
                        stdout_handle.join().expect("Failed to join stdout thread");
                        break;
                    }
                }
            }
        }
    }
    None
}

pub fn check_health(rx_task: Receiver<TaskEvent>) -> Option<String> {
    let mut child = match Command::new("brew")
        .arg("doctor")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return None,
    };

    let stderr = child.stderr.take().expect("no stdout");
    let (tx_output, rx_output) = channel::<String>();
    let stdout_handle = thread::spawn(move || {
        let mut output = String::new();
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(content) = line {
                output.push_str(&content);
                output.push('\n');
            }
        }
        tx_output.send(output).unwrap();
    });
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            let results = match rx_output.recv() {
                Ok(output) => output,
                _ => String::default(),
            };
            return Some(results);
        }
        if let Ok(task_event) = rx_task.try_recv() {
            match task_event {
                TaskEvent::Stop => {
                    child.kill().unwrap();
                    stdout_handle.join().expect("Failed to join stdout thread");
                    break;
                }
            }
        }
    }
    None
}

pub fn config(rx_task: Receiver<TaskEvent>) -> Option<String> {
    let mut child = match Command::new("brew")
        .arg("config")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return None,
    };

    let stdout = child.stdout.take().expect("no stdout");
    let (tx_output, rx_output) = channel::<String>();
    let stdout_handle = thread::spawn(move || {
        let mut output = String::new();
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(content) = line {
                output.push_str(&content);
                output.push('\n');
            }
        }
        tx_output.send(output).unwrap();
    });
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            let results = match rx_output.recv() {
                Ok(output) => output,
                _ => String::default(),
            };
            return Some(results);
        }
        if let Ok(task_event) = rx_task.try_recv() {
            match task_event {
                TaskEvent::Stop => {
                    child.kill().unwrap();
                    stdout_handle.join().expect("Failed to join stdout thread");
                    break;
                }
            }
        }
    }
    None
}

#[derive(Clone, Copy)]
pub enum PackageManager {
    Homebrew,
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Homebrew => write!(f, "Homebrew"),
        }
    }
}
//
// impl AsRef<OsStr> for PackageManager {
//     fn as_ref(&self) -> &OsStr {
//         match self {
//             Self::Homebrew => "brew".as_ref(),
//         }
//     }
// }

// let results = out
//     .split("\n")
//     .filter_map(|result| {
//         if !result.is_empty() && result == self.search_query {
//             Some(SearchResult {
//                 display_text: result.to_string(),
//             })
//         } else {
//             None
//         }
//     })
//     .collect::<Vec<_>>();
// if let Ok(search_query) = self.state.search.query.lock() {
//     if *search_query != self.search_query {
//         return;
//     }
// }
// if let Ok(mut search_results) = self.state.search.results.lock() {
//     if results.len() > 0 {
//         *search_results = results;
//     } else {
//         *search_results = Vec::new();
//     }
// }
