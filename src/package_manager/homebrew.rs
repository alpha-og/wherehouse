use std::sync::mpsc::Receiver;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use serde::Deserialize;

use crate::{fuzz, package_manager::error::PackageManagerError};

use super::{
    Backend, CommandResult, PackageManager, SearchResult, SpawnCommandResult, command,
    handle_spawned_command, spawn_command,
};

#[derive(Deserialize)]
struct BrewInfoOutput {
    formulae: Vec<FormulaInfo>,
    casks: Vec<CaskInfo>,
}

#[derive(Deserialize, Clone)]
struct FormulaInfo {
    name: String,
    desc: Option<String>,
    homepage: Option<String>,
    license: Option<String>,
    versions: Versions,
    installed: Vec<InstalledInfo>,
    linked_keg: Option<String>,
    outdated: Option<bool>,
    dependencies: Vec<String>,
    build_dependencies: Vec<String>,
    conflicts_with: Vec<String>,
    caveats: Option<String>,
    tap: Option<String>,
    deprecated: Option<bool>,
    deprecation_reason: Option<String>,
    disabled: Option<bool>,
    disable_reason: Option<String>,
}

#[derive(Deserialize, Clone)]
struct Versions {
    stable: Option<String>,
}

#[derive(Deserialize, Clone)]
struct InstalledInfo {
    version: String,
}

#[derive(Deserialize, Clone)]
struct CaskInfo {
    token: String,
    name: Vec<String>,
    desc: Option<String>,
    homepage: Option<String>,
    version: Option<String>,
    installed: Option<String>,
    outdated: Option<bool>,
    url: Option<String>,
}

pub struct Homebrew;

impl Homebrew {
    /// Display homebrew version
    fn brew_version() -> CommandResult {
        command(Backend::Homebrew.into(), ["--version"])
    }

    /// Install specified packages (casks/ formulae)
    fn brew_install<I, J>(options: Option<I>, package_list: J) -> SpawnCommandResult
    where
        I: IntoIterator<Item = InstallOption>,
        J: IntoIterator<Item = String>,
    {
        let mut args = vec!["install".to_string()];
        if let Some(options) = options {
            args.extend(
                options
                    .into_iter()
                    .map(|option: InstallOption| option.into()),
            );
        }
        args.extend(package_list);

        spawn_command(Backend::Homebrew, args)
    }

    /// Upgrade installed packages
    fn brew_upgrade<J>(package_list: Option<J>) -> SpawnCommandResult
    where
        J: IntoIterator<Item = String>,
    {
        let mut args = vec!["upgrade".to_string()];
        // if let Some(options) = options {
        //     args.extend(
        //         options
        //             .into_iter()
        //             .map(|option: InstallOption| option.into()),
        //     );
        // }
        if let Some(packages) = package_list {
            args.extend(packages);
        }

        spawn_command(Backend::Homebrew, args)
    }

    /// Uninstall specified packages (casks/ formulae)
    fn brew_uninstall<I, J>(options: Option<I>, package_list: J) -> SpawnCommandResult
    where
        I: IntoIterator<Item = UninstallOption>,
        J: IntoIterator<Item = String>,
    {
        let mut args = vec!["uninstall".to_string()];
        if let Some(options) = options {
            args.extend(
                options
                    .into_iter()
                    .map(|option: UninstallOption| option.into()),
            );
        }
        args.extend(package_list);

        spawn_command(Backend::Homebrew, args)
    }

    /// List installed packages (casks/ formulae)
    fn brew_list() -> CommandResult {
        command(Backend::Homebrew, ["list"])
    }

    /// Search homebrew core for specified pattern
    fn brew_search(pattern: String) -> CommandResult {
        command(Backend::Homebrew, ["search".to_string(), pattern])
    }

    /// Uninstall formulae that were only installed as a dependency
    /// of another formula and are now no longer needed
    fn brew_autoremove(dry_run: Option<AutoremoveOption>) -> CommandResult {
        if let Some(arg) = dry_run {
            command(Backend::Homebrew, ["autoremove", arg.into()])
        } else {
            command(Backend::Homebrew, ["autoremove"])
        }
    }
    /// List all locally installable casks including short names
    fn brew_casks() -> CommandResult {
        command(Backend::Homebrew, ["casks"])
    }

    /// List all locally installable formulae including short
    /// names
    fn brew_formulae() -> CommandResult {
        command(Backend::Homebrew, ["formulae"])
    }

    /// Remove stale lock files and outdated downloads for all
    /// formulae and casks, and remove old versions of installed
    /// formulae. If arguments are specified, only do this for
    /// the given formulae and casks. Removes all downloads
    /// more than 120 days old.
    fn brew_cleanup<I, J>(options: Option<I>, packages: Option<J>) -> CommandResult
    where
        I: IntoIterator<Item = CleanupOption>,
        J: IntoIterator<Item = String>,
    {
        let mut args = vec!["cleanup".to_string()];
        if let Some(options) = options {
            args.extend(
                options
                    .into_iter()
                    .map(|option: CleanupOption| option.into()),
            );
        };
        if let Some(packages) = packages {
            args.extend(packages.into_iter());
        }
        command(Backend::Homebrew, args)
    }

    /// Control whether Homebrew automatically links external
    /// tap shell completion files
    fn brew_completions(subcommand: Option<CompletionsSubcommand>) -> CommandResult {
        let mut args = vec!["completions"];
        if let Some(arg) = subcommand {
            args.push(arg.into());
        }
        command(Backend::Homebrew, args)
    }

    /// Show Homebrew and system configuration info useful
    /// for debugging
    fn brew_config() -> CommandResult {
        command(Backend::Homebrew, ["config"])
    }
    /// Display formula’s name and one-line description
    fn brew_desc<I, J>(options: Option<I>, query: Option<J>) -> SpawnCommandResult
    where
        I: IntoIterator<Item = DescOption>,
        J: IntoIterator<Item = String>,
    {
        let mut args = vec!["desc".to_string()];
        if let Some(options) = options {
            args.extend(options.into_iter().map(|option: DescOption| option.into()));
        }
        if let Some(query) = query {
            args.extend(query.into_iter());
        }

        spawn_command(Backend::Homebrew, args)
    }

    /// Check your system for potential problems
    fn brew_doctor<I>(options: Option<I>) -> SpawnCommandResult
    where
        I: IntoIterator<Item = DoctorOption>,
    {
        let mut args = vec!["doctor".to_string()];
        if let Some(options) = options {
            args.extend(
                options
                    .into_iter()
                    .map(|option: DoctorOption| option.into()),
            );
        }

        spawn_command(Backend::Homebrew, args)
    }

    /// Open a formula or cask’s homepage in a browser, or
    /// open Homebrew’s own homepage if no argument is provided
    fn brew_home<I>(options: Option<HomeOption>, query: Option<I>) -> CommandResult
    where
        I: IntoIterator<Item = String>,
    {
        let mut args = vec!["home".to_string()];

        if let Some(options) = options {
            args.push(options.into());
        }

        if let Some(query) = query {
            args.extend(query);
        }

        command(Backend::Homebrew, args)
    }

    /// Display brief statistics for your Homebrew installation
    ///
    /// If a formula or cask is provided, show summary of
    /// information about it
    fn brew_info<I, J>(options: Option<I>, query: Option<J>) -> SpawnCommandResult
    where
        I: IntoIterator<Item = InfoOption>,
        J: IntoIterator<Item = String>,
    {
        let mut args = vec!["info".to_string()];

        if let Some(options) = options {
            args.extend(options.into_iter().map(|option: InfoOption| option.into()));
        }

        if let Some(query) = query {
            args.extend(query);
        }

        spawn_command(Backend::Homebrew, args)
    }
}

impl PackageManager for Homebrew {
    fn alias(&self) -> &'static str {
        "brew"
    }

    fn filter_packages(
        &self,
        _rx: Receiver<bool>,
        pattern: String,
    ) -> Result<(Vec<SearchResult>, Option<String>), PackageManagerError> {
        let installed_output = Self::brew_list().map_err(PackageManagerError::from)?;
        let installed_names: Vec<String> = String::from_utf8(installed_output.stdout)
            .unwrap()
            .split('\n')
            .map(String::from)
            .filter(|s| !s.is_empty())
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

        let (remote_names, warning) = match Self::brew_search(pattern.clone()) {
            Ok(output) => {
                let names: Vec<String> = String::from_utf8(output.stdout)
                    .unwrap()
                    .split('\n')
                    .map(String::from)
                    .filter(|s| !s.is_empty())
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

        let matched = fuzz(all_names, pattern, None);

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

    fn package_manager_config(&self, _rx: Receiver<bool>) -> Result<String, PackageManagerError> {
        match Self::brew_config() {
            Ok(output) => Ok(String::from_utf8(output.stdout).unwrap()),
            Err(e) => Err(PackageManagerError::from(e)),
        }
    }

    fn package_info(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError> {
        let child = match Self::brew_info::<Vec<InfoOption>, _>(
            Some(vec![InfoOption::Json]),
            Some(vec![package_name]),
        ) {
            Ok(child) => child,
            Err(e) => return Err(PackageManagerError::from(e)),
        };
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }
    fn check_health(&self, rx: Receiver<bool>) -> Result<String, PackageManagerError> {
        let child = match Self::brew_doctor::<Vec<DoctorOption>>(None) {
            Ok(child) => child,
            Err(e) => return Err(PackageManagerError::from(e)),
        };
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.err.unwrap()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }
    fn clean(&self, _rx: Receiver<bool>) -> Result<String, PackageManagerError> {
        match Self::brew_cleanup::<Vec<CleanupOption>, Vec<String>>(None, None) {
            Ok(output) => Ok(String::from_utf8(output.stdout).unwrap()),
            Err(e) => Err(PackageManagerError::from(e)),
        }
    }
    fn install_package(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError> {
        let child = match Self::brew_install::<Vec<InstallOption>, _>(None, [package_name]) {
            Ok(child) => child,
            Err(e) => return Err(PackageManagerError::from(e)),
        };

        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }
    fn update_package(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError> {
        let child = match Self::brew_upgrade(Some([package_name])) {
            Ok(child) => child,
            Err(e) => return Err(PackageManagerError::from(e)),
        };

        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }

    fn update_all_packages(
        &self,
        rx: Receiver<bool>,
    ) -> Result<String, PackageManagerError> {
        let child = match Self::brew_upgrade(None::<[String; 0]>) {
            Ok(child) => child,
            Err(e) => return Err(PackageManagerError::from(e)),
        };
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }

    fn check_outdated(
        &self,
        _rx: Receiver<bool>,
    ) -> Result<Vec<String>, PackageManagerError> {
        let output = command(Backend::Homebrew, ["outdated"])
            .map_err(PackageManagerError::from)?;
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| PackageManagerError::ExecutionFailed(e.to_string()))?;
        let names: Vec<String> = stdout
            .lines()
            .filter_map(|line| line.split(' ').next())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        Ok(names)
    }

    fn uninstall_package(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError> {
        let child = match Self::brew_uninstall::<Vec<UninstallOption>, _>(None, [package_name]) {
            Ok(child) => child,
            Err(e) => return Err(PackageManagerError::from(e)),
        };

        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap()),
            None => Err(PackageManagerError::StaleCommand),
        }
    }

    fn render_info(&self, raw: &str) -> Vec<Line<'static>> {
        match serde_json::from_str::<BrewInfoOutput>(raw) {
            Ok(output) => {
                if !output.formulae.is_empty() {
                    build_formula_lines(output.formulae.into_iter().next().unwrap())
                } else if !output.casks.is_empty() {
                    build_cask_lines(output.casks.into_iter().next().unwrap())
                } else {
                    vec![Line::from(Span::raw(raw.to_string()))]
                }
            }
            Err(_) => vec![Line::from(Span::raw(raw.to_string()))],
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

        let kv: Vec<(String, String)> = raw
            .lines()
            .filter_map(|l| l.split_once(':').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
            .collect();

        lines.push(section_header("Backend"));
        lines.push(label_line("Name", "Homebrew"));
        for (k, v) in &kv {
            match k.as_str() {
                "HOMEBREW_VERSION" => lines.push(label_line("Version", v)),
                "HOMEBREW_PREFIX" => lines.push(label_line("Prefix", v)),
                "HOMEBREW_CELLAR" => lines.push(label_line("Cellar", v)),
                "HOMEBREW_REPOSITORY" => lines.push(label_line("Repository", v)),
                _ => {}
            }
        }
        lines.push(Line::from(""));

        lines.push(section_header("System"));
        for (k, v) in &kv {
            match k.as_str() {
                "BUILD_ARCH" => lines.push(label_line("Architecture", v)),
                "HOMEBREW_SYSTEM" | "MACOS_VERSION" => {}
                "HOMEBREW_CC" => lines.push(label_line("C Compiler", v)),
                "HOMEBREW_CXX" => lines.push(label_line("C++ Compiler", v)),
                "HOMEBREW_SYSTEM_VERSION" => lines.push(label_line("OS Version", v)),
                "SHELL" => lines.push(label_line("Shell", v)),
                _ => {}
            }
        }
        let os_name = kv.iter().find(|(k, _)| k == "HOMEBREW_SYSTEM").map(|(_, v)| v.as_str()).unwrap_or("");
        let os_ver = kv.iter().find(|(k, _)| k == "HOMEBREW_SYSTEM_VERSION").map(|(_, v)| v.as_str()).unwrap_or("");
        if !os_name.is_empty() {
            lines.push(label_line("OS", &format!("{os_name} {os_ver}")));
        }
        lines.push(Line::from(""));

        lines.push(section_header("Settings"));
        for (k, v) in &kv {
            if k.starts_with("HOMEBREW_") {
                if matches!(k.as_str(), "HOMEBREW_VERSION" | "HOMEBREW_PREFIX" | "HOMEBREW_CELLAR"
                    | "HOMEBREW_REPOSITORY" | "HOMEBREW_SHELLENV_PREFIX"
                    | "HOMEBREW_CC" | "HOMEBREW_CXX" | "HOMEBREW_SYSTEM"
                    | "HOMEBREW_SYSTEM_VERSION" | "HOMEBREW_BROWSER" | "HOMEBREW_EDITOR")
                {
                    continue;
                }
                let display_key = k.strip_prefix("HOMEBREW_").unwrap_or(k).to_string();
                if v == "1" {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(display_key, Style::default().fg(Color::Cyan)),
                        Span::raw("  "),
                        Span::styled("✓", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    ]));
                } else {
                    lines.push(label_line(&display_key, v));
                }
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

fn dep_list(deps: &[String]) -> Line<'static> {
    if deps.is_empty() {
        return Line::from(Span::raw(""));
    }
    Line::from(Span::raw(format!("    {}", deps.join(", "))))
}

fn build_formula_lines(f: FormulaInfo) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let mut title = format!("  {}", f.name);
    if let Some(ref ver) = f.versions.stable {
        title.push_str(&format!(" v{ver}"));
    }
    if !f.installed.is_empty() {
        title.push(' ');
        lines.push(Line::from(vec![
            Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "[installed]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
    }

    if let Some(ref desc) = f.desc {
        lines.push(Line::from(Span::styled(
            format!("  {desc}"),
            Style::default().fg(Color::DarkGray),
        )));
    }
    if let Some(ref hp) = f.homepage {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                hp.clone(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]));
    }

    lines.push(Line::from(""));

    let has_deps = !f.dependencies.is_empty() || !f.build_dependencies.is_empty();
    if has_deps {
        lines.push(section_header("Dependencies"));
        if !f.build_dependencies.is_empty() {
            lines.push(Line::from(Span::styled(
                "  Build:",
                Style::default().fg(Color::Cyan),
            )));
            lines.push(dep_list(&f.build_dependencies));
        }
        if !f.dependencies.is_empty() {
            lines.push(Line::from(Span::styled(
                "  Runtime:",
                Style::default().fg(Color::Cyan),
            )));
            lines.push(dep_list(&f.dependencies));
        }
        lines.push(Line::from(""));
    }

    lines.push(section_header("Details"));
    if let Some(ref lic) = f.license {
        lines.push(label_line("License", lic));
    }
    if let Some(ref tap) = f.tap {
        lines.push(label_line("Tap", tap));
    }
    if let Some(ref keg) = f.linked_keg {
        lines.push(label_line("Linked", keg));
    }
    if let Some(true) = f.outdated {
        lines.push(Line::from(Span::styled(
            "  Outdated",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }
    if let Some(true) = f.deprecated {
        let reason = f.deprecation_reason.unwrap_or_else(|| "unknown".into());
        lines.push(Line::from(Span::styled(
            format!("  Deprecated: {reason}"),
            Style::default().fg(Color::Red),
        )));
    }
    if let Some(true) = f.disabled {
        let reason = f.disable_reason.unwrap_or_else(|| "unknown".into());
        lines.push(Line::from(Span::styled(
            format!("  Disabled: {reason}"),
            Style::default().fg(Color::Red),
        )));
    }
    if !f.conflicts_with.is_empty() {
        lines.push(label_line("Conflicts", &f.conflicts_with.join(", ")));
    }
    if let Some(ref msg) = f.caveats {
        lines.push(Line::from(""));
        lines.push(section_header("Caveats"));
        for line in msg.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {line}"),
                Style::default().fg(Color::Yellow),
            )));
        }
    }

    lines
}

fn build_cask_lines(c: CaskInfo) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let display_name = c.name.first().map(|s| s.as_str()).unwrap_or(&c.token);

    let mut title = format!("  {display_name}");
    if let Some(ref ver) = c.version {
        title.push_str(&format!(" v{ver}"));
    }
    if c.installed.is_some() {
        title.push(' ');
        lines.push(Line::from(vec![
            Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "[installed]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
    }

    if let Some(ref desc) = c.desc {
        lines.push(Line::from(Span::styled(
            format!("  {desc}"),
            Style::default().fg(Color::DarkGray),
        )));
    }
    if let Some(ref hp) = c.homepage {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                hp.clone(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(section_header("Details"));
    if let Some(ref ver) = c.version {
        lines.push(label_line("Version", ver));
    }
    if let Some(ref installed) = c.installed {
        lines.push(label_line("Installed", installed));
    }
    if let Some(true) = c.outdated {
        lines.push(Line::from(Span::styled(
            "  Outdated",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }
    if let Some(ref url) = c.url {
        lines.push(label_line("URL", url));
    }

    lines
}

pub enum AutoremoveOption {
    DryRun,
}

impl From<AutoremoveOption> for &'static str {
    fn from(value: AutoremoveOption) -> Self {
        match value {
            AutoremoveOption::DryRun => "--dry-run",
        }
    }
}

pub enum CleanupOption {
    Prune,
    DryRun,
    Scrub,
    PrunePrefix,
}

impl From<CleanupOption> for String {
    fn from(value: CleanupOption) -> Self {
        match value {
            CleanupOption::Prune => "--prune".to_string(),
            CleanupOption::DryRun => "--dry-run".to_string(),
            CleanupOption::Scrub => "--scrub".to_string(),
            CleanupOption::PrunePrefix => "--prune-prefix".to_string(),
        }
    }
}

pub enum CompletionsSubcommand {
    Link,
    Unlink,
}

impl From<CompletionsSubcommand> for &'static str {
    fn from(value: CompletionsSubcommand) -> Self {
        match value {
            CompletionsSubcommand::Link => "link",
            CompletionsSubcommand::Unlink => "unlink",
        }
    }
}

pub enum DescOption {
    Search,
    Name,
    Description,
    EvalAll,
    Formula,
    Cask,
}

impl From<DescOption> for String {
    fn from(value: DescOption) -> Self {
        match value {
            DescOption::Search => "--search".to_string(),
            DescOption::Name => "--name".to_string(),
            DescOption::Description => "--description".to_string(),
            DescOption::EvalAll => "--eval-all".to_string(),
            DescOption::Formula => "--formula".to_string(),
            DescOption::Cask => "--cask".to_string(),
        }
    }
}

pub enum DoctorOption {
    ListChecks,
    AuditDebug,
}

impl From<DoctorOption> for String {
    fn from(value: DoctorOption) -> Self {
        match value {
            DoctorOption::ListChecks => "--list-checks".to_string(),
            DoctorOption::AuditDebug => "--audit-debug".to_string(),
        }
    }
}

pub enum HomeOption {
    Formula,
    Cask,
}

impl From<HomeOption> for String {
    fn from(value: HomeOption) -> Self {
        match value {
            HomeOption::Formula => "--formula".to_string(),
            HomeOption::Cask => "--cask".to_string(),
        }
    }
}

pub enum InfoOption {
    Analytics,
    Days,
    Category,
    Github,
    FetchManifest,
    Json,
    Installed,
    EvalAll,
    Variations,
    Verbose,
    Formula,
    Cask,
}

impl From<InfoOption> for String {
    fn from(value: InfoOption) -> Self {
        match value {
            InfoOption::Analytics => "--analytics".to_string(),
            InfoOption::Days => "--days".to_string(),
            InfoOption::Category => "--category".to_string(),
            InfoOption::Github => "--github".to_string(),
            InfoOption::FetchManifest => "--fetch-manifest".to_string(),
            InfoOption::Json => "--json=v2".to_string(),
            InfoOption::Installed => "--installed".to_string(),
            InfoOption::EvalAll => "--eval-all".to_string(),
            InfoOption::Variations => "--variations".to_string(),
            InfoOption::Verbose => "--verbose".to_string(),
            InfoOption::Formula => "--formula".to_string(),
            InfoOption::Cask => "--cask".to_string(),
        }
    }
}

pub enum InstallOption {
    Debug,
    DisplayTimes,
    Force,
    Verbose,
    DryRun,
    Ask,
    Formula,
    IgnoreDependencies,
    OnlyDependencies,
    Cc,
    BuildFromSource,
    ForceBottle,
    IncludeTest,
    Head,
    FetchHead,
    KeepTmp,
    DebugSymbols,
    BuildBottle,
    SkipPostInstall,
    SkipLink,
    AsDependency,
    BottleArch,
    Interactive,
    Git,
    Overwrite,
    Cask,
    NoBinaries,
    Binaries,
    RequireSHA,
    Quarantine,
    Adopt,
    SkipCaskDeps,
    Zap,
}

impl From<InstallOption> for String {
    fn from(value: InstallOption) -> Self {
        match value {
            InstallOption::Debug => "--debug".to_string(),
            InstallOption::DisplayTimes => "--display-times".to_string(),
            InstallOption::Force => "--force".to_string(),
            InstallOption::Verbose => "--verbose".to_string(),
            InstallOption::DryRun => "--dry-run".to_string(),
            InstallOption::Ask => "--ask".to_string(),
            InstallOption::Formula => "--formula".to_string(),
            InstallOption::IgnoreDependencies => "--ignore-dependencies".to_string(),
            InstallOption::OnlyDependencies => "--only-dependencies".to_string(),
            InstallOption::Cc => "--cc".to_string(),
            InstallOption::BuildFromSource => "--build-from-source".to_string(),
            InstallOption::ForceBottle => "--force-bottle".to_string(),
            InstallOption::IncludeTest => "--include-test".to_string(),
            InstallOption::Head => "--HEAD".to_string(),
            InstallOption::FetchHead => "--fetch-head".to_string(),
            InstallOption::KeepTmp => "--keep-tmp".to_string(),
            InstallOption::DebugSymbols => "--debug-symbols".to_string(),
            InstallOption::BuildBottle => "--build-bottle".to_string(),
            InstallOption::SkipPostInstall => "--skip-post-install".to_string(),
            InstallOption::SkipLink => "--skip-link".to_string(),
            InstallOption::AsDependency => "--as-dependency".to_string(),
            InstallOption::BottleArch => "--bottle-arch".to_string(),
            InstallOption::Interactive => "--interactive".to_string(),
            InstallOption::Git => "--git".to_string(),
            InstallOption::Overwrite => "--overwrite".to_string(),
            InstallOption::Cask => "--cask".to_string(),
            InstallOption::NoBinaries => "--no-binaries".to_string(),
            InstallOption::Binaries => "--binaries".to_string(),
            InstallOption::RequireSHA => "--require-sha".to_string(),
            InstallOption::Quarantine => "--quarantine".to_string(),
            InstallOption::Adopt => "--adopt".to_string(),
            InstallOption::SkipCaskDeps => "--skip-cask-deps".to_string(),
            InstallOption::Zap => "--zap".to_string(),
        }
    }
}

pub enum UninstallOption {
    Force,
    Zap,
    IgnoreDependencies,
    Formula,
    Cask,
}

impl From<UninstallOption> for String {
    fn from(value: UninstallOption) -> Self {
        match value {
            UninstallOption::Force => "--force".to_string(),
            UninstallOption::Zap => "--zap".to_string(),
            UninstallOption::IgnoreDependencies => "--ignore-dependencies".to_string(),
            UninstallOption::Formula => "--formula".to_string(),
            UninstallOption::Cask => "--cask".to_string(),
        }
    }
}
