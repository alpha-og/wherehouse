use std::env;

use super::manager::PackageManager;

use crate::package_manager::homebrew;

#[derive(Debug, Clone)]
pub enum Backend {
    Homebrew,
    Pacman,
    Yay,
    Dnf,
    Apt,
    Winget,
}

impl Default for Backend {
    fn default() -> Self {
        Self::available().get(0).take().unwrap().clone()
    }
}

impl From<&str> for Backend {
    fn from(value: &str) -> Self {
        match value {
            "homebrew" | "brew" => Backend::Homebrew,
            _ => Backend::Homebrew,
        }
    }
}

impl Backend {
    pub fn alias(&self) -> &'static str {
        match self {
            Backend::Homebrew => "brew",
            Backend::Pacman => "pacman",
            Backend::Yay => "yay",
            Backend::Dnf => "dnf",
            Backend::Apt => "apt",
            Backend::Winget => "winget",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Backend::Homebrew => "Homebrew",
            Backend::Pacman => "Pacman",
            Backend::Yay => "Yet Another Yogurt",
            Backend::Dnf => "Dandified YUM",
            Backend::Apt => "Advanced Package Tool",
            Backend::Winget => "Windows Package Manager",
        }
    }

    pub fn package_manager_from_backend(backend: Backend) -> impl PackageManager {
        match backend {
            Backend::Homebrew => homebrew::Homebrew,
            _ => panic!("Unsupported package manager"),
        }
    }

    fn is_installed(&self, split_paths: env::SplitPaths) -> bool {
        split_paths
            .filter(|path| {
                let path_to_binary = path.join(self.alias());
                path_to_binary.is_file()
            })
            .collect::<Vec<_>>()
            .len()
            != 0
    }

    fn available() -> Vec<Backend> {
        let mut available_backends = vec![];
        let path = match env::var_os("PATH") {
            Some(path) => path,
            None => return available_backends,
        };
        let supported_backends = [
            Backend::Homebrew,
            Backend::Pacman,
            Backend::Yay,
            Backend::Dnf,
            Backend::Apt,
            Backend::Winget,
        ];

        supported_backends.iter().for_each(|backend| {
            let split_paths = env::split_paths(&path);
            if backend.is_installed(split_paths) {
                available_backends.push(backend.clone());
            }
        });

        available_backends
    }
}
