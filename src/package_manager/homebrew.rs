use std::sync::mpsc::Receiver;

use super::{
    CommandResult, PackageLocality, PackageManager, SpawnCommandResult, command,
    handle_spawned_command, spawn_command,
};

pub struct Homebrew;

const HOMEBREW_ALIAS: &'static str = "brew";

impl Homebrew {
    /// Display homebrew version
    fn brew_version() -> CommandResult {
        command(HOMEBREW_ALIAS, ["--version"])
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

        spawn_command(HOMEBREW_ALIAS, args)
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

        spawn_command(HOMEBREW_ALIAS, args)
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

        spawn_command(HOMEBREW_ALIAS, args)
    }

    /// List installed packages (casks/ formulae)
    fn brew_list() -> CommandResult {
        command(HOMEBREW_ALIAS, ["list"])
    }

    /// Search homebrew core for specified pattern
    fn brew_search(pattern: String) -> CommandResult {
        command(HOMEBREW_ALIAS, ["search".to_string(), pattern])
    }

    /// Uninstall formulae that were only installed as a dependency
    /// of another formula and are now no longer needed
    fn brew_autoremove(dry_run: Option<AutoremoveOption>) -> CommandResult {
        if let Some(arg) = dry_run {
            command(HOMEBREW_ALIAS, ["autoremove", arg.into()])
        } else {
            command(HOMEBREW_ALIAS, ["autoremove"])
        }
    }
    /// List all locally installable casks including short names
    fn brew_casks() -> CommandResult {
        command(HOMEBREW_ALIAS, ["casks"])
    }

    /// List all locally installable formulae including short
    /// names
    fn brew_formulae() -> CommandResult {
        command(HOMEBREW_ALIAS, ["formulae"])
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
        command(HOMEBREW_ALIAS, args)
    }

    /// Control whether Homebrew automatically links external
    /// tap shell completion files
    fn brew_completions(subcommand: Option<CompletionsSubcommand>) -> CommandResult {
        let mut args = vec!["completions"];
        if let Some(arg) = subcommand {
            args.push(arg.into());
        }
        command(HOMEBREW_ALIAS, args)
    }

    /// Show Homebrew and system configuration info useful
    /// for debugging
    fn brew_config() -> CommandResult {
        command(HOMEBREW_ALIAS, ["config"])
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

        spawn_command(HOMEBREW_ALIAS, args)
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

        spawn_command(HOMEBREW_ALIAS, args)
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

        command(HOMEBREW_ALIAS, args)
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

        spawn_command(HOMEBREW_ALIAS, args)
    }
}

impl PackageManager for Homebrew {
    fn alias(&self) -> &'static str {
        "brew"
    }
    fn filter_packages(
        &self,
        _rx: Receiver<bool>,
        package_locality: super::PackageLocality,
        pattern: String,
    ) -> Result<Vec<String>, String> {
        match package_locality {
            PackageLocality::Local => match Self::brew_list() {
                Ok(output) => Ok(String::from_utf8(output.stdout)
                    .unwrap()
                    .split("\n")
                    .map(|item| item.to_string())
                    .collect::<Vec<String>>()),
                Err(e) => Err(format!("failed to execute command brew list: {e}")),
            },
            PackageLocality::Remote => match Self::brew_search(pattern) {
                Ok(output) => Ok(String::from_utf8(output.stdout)
                    .unwrap()
                    .split("\n")
                    .map(|item| item.to_string())
                    .collect::<Vec<String>>()),
                Err(e) => Err(format!("failed to execute command brew list: {e}")),
            },
        }
    }

    fn package_info(&self, rx: Receiver<bool>, package_name: String) -> Result<String, String> {
        let child = match Self::brew_desc(
            Some([DescOption::Search, DescOption::EvalAll]),
            Some([package_name]),
        ) {
            Ok(child) => child,
            Err(e) => return Err(format!("{e}")),
        };
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.out.unwrap()),
            None => Err("could not execute command".to_string()),
        }
    }
    fn check_health(&self, rx: Receiver<bool>) -> Result<String, String> {
        let child = match Self::brew_doctor::<Vec<DoctorOption>>(None) {
            Ok(child) => child,
            Err(e) => return Err(format!("{e}")),
        };
        match handle_spawned_command(rx, child) {
            Some(output) => Ok(output.err.unwrap()),
            None => Err("could not execute command".to_string()),
        }
    }
    fn clean(&self, rx: Receiver<bool>) -> Result<String, String> {
        match Self::brew_cleanup::<Vec<CleanupOption>, Vec<String>>(None, None) {
            Ok(output) => Ok(String::from_utf8(output.stdout).unwrap()),
            Err(e) => Err(format!("{e}")),
        }
    }
    fn install_package(&self, rx: Receiver<bool>, package_name: String) -> Result<(), String> {
        let child = match Self::brew_install::<Vec<InstallOption>, _>(None, [package_name]) {
            Ok(child) => child,
            Err(e) => return Err(format!("{e}")),
        };

        match handle_spawned_command(rx, child) {
            Some(_) => Ok(()),
            None => Err("could not execute command".to_string()),
        }
    }
    fn update_package(&self, rx: Receiver<bool>, package_name: String) -> Result<(), String> {
        let child = match Self::brew_upgrade(Some([package_name])) {
            Ok(child) => child,
            Err(e) => return Err(format!("{e}")),
        };

        match handle_spawned_command(rx, child) {
            Some(_) => Ok(()),
            None => Err("could not execute command".to_string()),
        }
    }
    fn uninstall_package(&self, rx: Receiver<bool>, package_name: String) -> Result<(), String> {
        let child = match Self::brew_uninstall::<Vec<UninstallOption>, _>(None, [package_name]) {
            Ok(child) => child,
            Err(e) => return Err(format!("{e}")),
        };

        match handle_spawned_command(rx, child) {
            Some(_) => Ok(()),
            None => Err("could not execute command".to_string()),
        }
    }
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
            InfoOption::Json => "--json".to_string(),
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
