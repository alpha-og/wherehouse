mod homebrew;

pub type SpawnCommandResult = Result<std::process::Child, std::io::Error>;
pub type CommandResult = std::io::Result<std::process::Output>;

// enum PackageManager {
//     Homebrew,
//     Pacman,
//     Apt,
//     Winget,
// }
//
// type Alias = &'static str;
//
// impl From<PackageManager> for Alias {
//     fn from(value: PackageManager) -> Self {
//         match value {
//             PackageManager::Homebrew => "brew",
//             PackageManager::Pacman => "pacman",
//             PackageManager::Apt => "apt",
//             PackageManager::Winget => "winget",
//         }
//     }
// }
//
//

