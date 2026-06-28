use std::sync::mpsc::Receiver;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

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

    fn check_outdated(
        &self,
        _rx: Receiver<bool>,
    ) -> Result<Vec<String>, PackageManagerError> {
        Err(PackageManagerError::UnsupportedOperation(
            "check outdated".into(),
        ))
    }

    fn update_all_packages(
        &self,
        _rx: Receiver<bool>,
    ) -> Result<String, PackageManagerError> {
        Err(PackageManagerError::UnsupportedOperation(
            "update all".into(),
        ))
    }

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

    fn render_info(&self, raw: &str) -> Vec<Line<'static>> {
        vec![Line::from(Span::raw(raw.to_string()))]
    }

    fn render_config(&self, raw: &str, app_name: &str, app_version: &str) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                format!("  {app_name} v{app_version}"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "  ─────────────────────",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::raw(raw.to_string())),
        ]
    }
}
