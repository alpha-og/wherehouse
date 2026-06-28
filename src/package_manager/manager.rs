use std::sync::mpsc::Receiver;

use super::error::PackageManagerError;
use super::types::SearchResult;

pub trait PackageManager: Send + Sync + 'static {
    fn alias(&self) -> &'static str;
    fn filter_packages(
        &self,
        rx: Receiver<bool>,
        pattern: String,
    ) -> Result<(Vec<SearchResult>, Option<String>), PackageManagerError>;
    fn install_package(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError>;
    fn update_package(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError>;
    fn uninstall_package(
        &self,
        rx: Receiver<bool>,
        package_name: String,
    ) -> Result<String, PackageManagerError>;

    fn package_manager_config(
        &self,
        _rx: Receiver<bool>,
    ) -> Result<String, PackageManagerError> {
        Err(PackageManagerError::UnsupportedOperation("config".into()))
    }

    fn package_info(
        &self,
        _rx: Receiver<bool>,
        _package_name: String,
    ) -> Result<String, PackageManagerError> {
        Err(PackageManagerError::UnsupportedOperation("info".into()))
    }

    fn check_health(&self, _rx: Receiver<bool>) -> Result<String, PackageManagerError> {
        Err(PackageManagerError::UnsupportedOperation(
            "health check".into(),
        ))
    }

    fn clean(&self, _rx: Receiver<bool>) -> Result<String, PackageManagerError> {
        Err(PackageManagerError::UnsupportedOperation("clean".into()))
    }
}
