#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Command {
    FilterPackages,
    Config,
    PackageInfo,
    GeneralInfo,
    CheckHealth,
    InstallPackage,
    UninstallPackage,
    UpdatePackage,
    UpdateAll,
    Clean,
    CheckOutdated,
}
