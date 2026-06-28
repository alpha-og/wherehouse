use std::sync::mpsc::Receiver;

use crate::package_manager::error::PackageManagerError;

use super::{
    Backend, PackageManager, SearchResult,
    command, handle_spawned_command, spawn_command,
};

pub struct Dnf;

impl Dnf {
    fn dnf_list_installed() -> super::CommandResult {
        command(Backend::Dnf, ["list", "installed"])
    }

    fn dnf_search(pattern: String) -> super::CommandResult {
        command(Backend::Dnf, ["search".to_string(), pattern])
    }

    fn dnf_install(package: String) -> super::SpawnCommandResult {
        spawn_command(Backend::Dnf, vec!["install", "-y", package.as_str()])
    }

    fn dnf_remove(package: String) -> super::SpawnCommandResult {
        spawn_command(Backend::Dnf, vec!["remove", "-y", package.as_str()])
    }

    fn dnf_upgrade(package: Option<String>) -> super::SpawnCommandResult {
        let mut args = vec!["upgrade", "-y"];
        if let Some(pkg) = &package {
            args.push(pkg.as_str());
        }
        spawn_command(Backend::Dnf, args)
    }

    fn dnf_check_update() -> super::CommandResult {
        command(Backend::Dnf, ["check-update"])
    }

    fn dnf_repolist() -> super::CommandResult {
        command(Backend::Dnf, ["repolist"])
    }

    fn dnf_clean_all() -> super::CommandResult {
        command(Backend::Dnf, ["clean", "all"])
    }
}

fn strip_arch(name: &str) -> &str {
    if let Some(dot_pos) = name.rfind('.') {
        let arch = &name[dot_pos + 1..];
        if !arch.is_empty()
            && arch.len() <= 7
            && arch.chars().all(|c| c.is_ascii_alphanumeric())
        {
            return &name[..dot_pos];
        }
    }
    name
}

fn parse_package_names(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let first = line.split_whitespace().next()?;
            let name = strip_arch(first);
            if name.is_empty() {
                return None;
            }
            Some(name.to_string())
        })
        .collect()
}

impl PackageManager for Dnf {
    fn alias(&self) -> &'static str {
        "dnf"
    }

    fn filter_packages(
        &self,
        _rx: Receiver<bool>,
        pattern: String,
    ) -> Result<(Vec<SearchResult>, Option<String>), PackageManagerError> {
        let installed_output = Self::dnf_list_installed().map_err(PackageManagerError::from)?;
        let installed_stdout = String::from_utf8(installed_output.stdout).unwrap_or_default();

        let installed_names: Vec<String> = installed_stdout
            .lines()
            .skip_while(|line| !line.starts_with("Installed "))
            .skip(1)
            .flat_map(|line| {
                let first = line.split_whitespace().next()?;
                let name = strip_arch(first);
                if name.is_empty() { None } else { Some(name.to_string()) }
            })
            .collect();

        let installed_set: std::collections::HashSet<String> =
            installed_names.iter().cloned().collect();

        if pattern.is_empty() {
            let results: Vec<SearchResult> = installed_names
                .into_iter()
                .map(|name| SearchResult {
                    is_installed: true,
                    update_available: false,
                    name,
                })
                .collect();
            return Ok((results, None));
        }

        let (remote_names, warning) = match Self::dnf_search(pattern.clone()) {
            Ok(output) => {
                let stdout = String::from_utf8(output.stdout).unwrap_or_default();
                let names: Vec<String> = stdout
                    .lines()
                    .filter(|line| line.contains(" : "))
                    .filter_map(|line| {
                        let first = line.splitn(2, " : ").next()?;
                        let name = strip_arch(first);
                        if name.is_empty() { None } else { Some(name.to_string()) }
                    })
                    .collect();
                (names, None)
            }
            Err(e) => (Vec::new(), Some(format!("Remote search failed: {e}"))),
        };

        let mut seen = std::collections::HashSet::new();
        let all_names: Vec<String> = installed_names
            .into_iter()
            .chain(remote_names.into_iter())
            .filter(|n| seen.insert(n.clone()))
            .collect();

        let matched = crate::fuzz(all_names, pattern, None);

        let results: Vec<SearchResult> = matched
            .into_iter()
            .map(|name| SearchResult {
                is_installed: installed_set.contains(&name),
                update_available: false,
                name,
            })
            .collect();

        Ok((results, warning))
    }

    fn install_package(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError> {
        let child = Self::dnf_install(package_name)?;
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap_or_default()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }

    fn uninstall_package(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError> {
        let child = Self::dnf_remove(package_name)?;
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap_or_default()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }

    fn update_package(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError> {
        let child = Self::dnf_upgrade(Some(package_name))?;
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap_or_default()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }

    fn update_all_packages(
        &self,
        rx: Receiver<bool>,
    ) -> Result<String, PackageManagerError> {
        let child = Self::dnf_upgrade(None)?;
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap_or_default()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }

    fn check_outdated(
        &self,
        _rx: Receiver<bool>,
    ) -> Result<Vec<String>, PackageManagerError> {
        let output = Self::dnf_check_update().map_err(PackageManagerError::from)?;
        let stdout = String::from_utf8(output.stdout).unwrap_or_default();
        let stderr = String::from_utf8_lossy(&output.stderr);

        let combined = if stdout.is_empty() { stderr.as_ref() } else { stdout.as_str() };

        let names = parse_package_names(combined);
        Ok(names)
    }

    fn package_info(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError> {
        let child = spawn_command(Backend::Dnf, vec!["info", package_name.as_str()])?;
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap_or_default()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }

    fn package_manager_config(&self, _rx: Receiver<bool>) -> Result<String, PackageManagerError> {
        match Self::dnf_repolist() {
            Ok(output) => Ok(String::from_utf8(output.stdout).unwrap_or_default()),
            Err(e) => Err(PackageManagerError::from(e)),
        }
    }

    fn check_health(&self, _rx: Receiver<bool>) -> Result<String, PackageManagerError> {
        match Self::dnf_repolist() {
            Ok(output) => {
                let stdout = String::from_utf8(output.stdout).unwrap_or_default();
                if output.status.success() {
                    Ok(stdout)
                } else {
                    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
                    Err(PackageManagerError::ExecutionFailed(stderr))
                }
            }
            Err(e) => Err(PackageManagerError::from(e)),
        }
    }

    fn clean(&self, _rx: Receiver<bool>) -> Result<String, PackageManagerError> {
        match Self::dnf_clean_all() {
            Ok(output) => Ok(String::from_utf8(output.stdout).unwrap_or_default()),
            Err(e) => Err(PackageManagerError::from(e)),
        }
    }
}
