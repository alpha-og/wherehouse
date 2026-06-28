use std::sync::mpsc::Receiver;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::package_manager::error::PackageManagerError;

use super::{
    Backend, PackageManager, SearchResult,
    command, handle_spawned_command, spawn_command,
};

pub struct Pnpm;

impl Pnpm {
    fn pnpm_list_global() -> super::CommandResult {
        command(Backend::Pnpm, ["list", "-g", "--depth=0", "--json"])
    }

    fn pnpm_search(pattern: String) -> super::CommandResult {
        command(Backend::Pnpm, ["search".to_string(), pattern])
    }

    fn pnpm_add_global(package: String) -> super::SpawnCommandResult {
        spawn_command(Backend::Pnpm, vec!["add", "-g", package.as_str()])
    }

    fn pnpm_remove_global(package: String) -> super::SpawnCommandResult {
        spawn_command(Backend::Pnpm, vec!["remove", "-g", package.as_str()])
    }

    fn pnpm_update_global(package: Option<String>) -> super::SpawnCommandResult {
        let mut args = vec!["update", "-g"];
        if let Some(pkg) = &package {
            args.push(pkg.as_str());
        }
        spawn_command(Backend::Pnpm, args)
    }

    fn pnpm_outdated_global() -> super::CommandResult {
        command(Backend::Pnpm, ["outdated", "-g", "--json"])
    }

    fn pnpm_config_list() -> super::CommandResult {
        command(Backend::Pnpm, ["config", "list"])
    }

    fn pnpm_store_prune() -> super::CommandResult {
        command(Backend::Pnpm, ["store", "prune"])
    }

    fn pnpm_version() -> super::CommandResult {
        command(Backend::Pnpm, ["--version"])
    }
}

fn parse_installed_from_json(stdout: &str) -> Vec<String> {
    let parsed: serde_json::Value = match serde_json::from_str(stdout) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut names = Vec::new();
    for key in &["dependencies", "devDependencies", "optionalDependencies"] {
        if let Some(deps) = parsed.get(*key).and_then(|v| v.as_object()) {
            names.extend(deps.keys().cloned());
        }
    }
    names
}

fn parse_installed_from_text(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .skip_while(|line| !line.trim().ends_with("dependencies:"))
        .skip(1)
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let name = trimmed.split_whitespace().next()?;
            if name.contains('/') || name.starts_with('(') {
                return None;
            }
            Some(name.to_string())
        })
        .collect()
}

impl PackageManager for Pnpm {
    fn alias(&self) -> &'static str {
        "pnpm"
    }

    fn filter_packages(
        &self,
        _rx: Receiver<bool>,
        pattern: String,
    ) -> Result<(Vec<SearchResult>, Option<String>), PackageManagerError> {
        let installed_output = Self::pnpm_list_global().map_err(PackageManagerError::from)?;
        let installed_stdout = String::from_utf8(installed_output.stdout).unwrap_or_default();

        let installed_names = if installed_output.status.success() {
            let from_json = parse_installed_from_json(&installed_stdout);
            if !from_json.is_empty() {
                from_json
            } else {
                parse_installed_from_text(&installed_stdout)
            }
        } else {
            Vec::new()
        };

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

        let (remote_names, warning) = match Self::pnpm_search(pattern.clone()) {
            Ok(output) => {
                let stdout = String::from_utf8(output.stdout).unwrap_or_default();
                let names: Vec<String> = stdout
                    .lines()
                    .filter(|line| line.contains('|') && !line.starts_with('┌'))
                    .filter_map(|line| {
                        let name = line.split('|').next()?.trim();
                        if name.is_empty() {
                            return None;
                        }
                        Some(name.to_string())
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
        let child = Self::pnpm_add_global(package_name)?;
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
        let child = Self::pnpm_remove_global(package_name)?;
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
        let child = Self::pnpm_update_global(Some(package_name))?;
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap_or_default()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }

    fn update_all_packages(
        &self,
        rx: Receiver<bool>,
    ) -> Result<String, PackageManagerError> {
        let child = Self::pnpm_update_global(None)?;
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap_or_default()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }

    fn check_outdated(
        &self,
        _rx: Receiver<bool>,
    ) -> Result<Vec<String>, PackageManagerError> {
        let output = Self::pnpm_outdated_global().map_err(PackageManagerError::from)?;
        let stdout = String::from_utf8(output.stdout).unwrap_or_default();

        let names: Vec<String> =
            if let Some(obj) = serde_json::from_str::<std::collections::BTreeMap<String, serde_json::Value>>(&stdout).ok() {
                obj.into_keys().collect()
            } else {
                stdout
                    .lines()
                    .filter(|line| {
                        !line.contains('┌')
                            && !line.contains('┐')
                            && !line.contains('┤')
                            && !line.contains('├')
                            && !line.contains('└')
                            && !line.contains('┘')
                            && !line.contains("Package")
                    })
                    .filter_map(|line| {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            return None;
                        }
                        let name = if let Some(pipe_idx) = trimmed.find('│') {
                            trimmed[..pipe_idx].trim()
                        } else {
                            trimmed.split_whitespace().next()?
                        };
                        if name.is_empty() {
                            return None;
                        }
                        Some(name.to_string())
                    })
                    .collect()
            };

        Ok(names)
    }

    fn package_info(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError> {
        let child = spawn_command(Backend::Pnpm, vec!["info", package_name.as_str()])?;
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap_or_default()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }

    fn package_manager_config(&self, _rx: Receiver<bool>) -> Result<String, PackageManagerError> {
        match Self::pnpm_config_list() {
            Ok(output) => Ok(String::from_utf8(output.stdout).unwrap_or_default()),
            Err(e) => Err(PackageManagerError::from(e)),
        }
    }

    fn check_health(&self, _rx: Receiver<bool>) -> Result<String, PackageManagerError> {
        match Self::pnpm_version() {
            Ok(output) => {
                let stdout = String::from_utf8(output.stdout).unwrap_or_default();
                if output.status.success() {
                    Ok(stdout.trim().to_string())
                } else {
                    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
                    Err(PackageManagerError::ExecutionFailed(stderr))
                }
            }
            Err(e) => Err(PackageManagerError::from(e)),
        }
    }

    fn clean(&self, _rx: Receiver<bool>) -> Result<String, PackageManagerError> {
        match Self::pnpm_store_prune() {
            Ok(output) => Ok(String::from_utf8(output.stdout).unwrap_or_default()),
            Err(e) => Err(PackageManagerError::from(e)),
        }
    }

    fn render_config(&self, raw: &str, app_name: &str, app_version: &str) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled(
            format!("  {app_name} v{app_version}"),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  ─────────────────────",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        lines.push(section_header("Backend"));
        lines.push(label_line("Name", "pnpm"));

        let kv: Vec<_> = raw
            .lines()
            .filter_map(|l| {
                let l = l.trim();
                if l.is_empty() || l.starts_with(';') {
                    return None;
                }
                l.split_once('=').map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            })
            .collect();

        if !kv.is_empty() {
            lines.push(Line::from(""));
            lines.push(section_header("Config"));
            for (k, v) in &kv {
                lines.push(label_line(k, v));
            }
        }

        lines
    }
}

fn section_header(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ── {} ──", text),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
}

fn label_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {}: ", label),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(value.to_string()),
    ])
}
