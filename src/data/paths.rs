//! **paths** is a module that contains application-wide path definitions
//!
//! Evaluates OS-specific paths lazily at runtime for configuration, binaries, data, UIs, and app assets.
//! These paths are shared by the backend, the Launcher and every GUI, so they must never
//! be redefined elsewhere: depend on this crate (or run `ATA --files`) instead.
//!
//! Main static: [`PATHS`]

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use serde::Serialize;

/// Container struct for all paths used by ATA across supported operating systems
#[derive(Serialize, Clone, Debug)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Paths {
    /// Path to the primary executable binary (the `ATA` CLI)
    pub executable: PathBuf,
    /// Path to `data.json` storing runtime mod state
    pub data_file: PathBuf,
    /// Path to `settings.json` storing user configurations
    pub settings_file: PathBuf,
    /// Folder containing the installed GUIs of kind `Webapp`
    pub uis_dir: PathBuf,
    /// Folder containing the installed GUIs of kind `App`
    pub apps_dir: PathBuf,
    /// The user's downloads folder (falls back to the home folder)
    pub downloads_dir: PathBuf,
}

impl Paths {
    /// Resolves every path for the current user and OS
    ///
    /// # Panics
    /// Panics if the OS doesn't expose a local data or config directory,
    /// which only happens on a broken user environment (e.g. no `$HOME`)
    pub fn new() -> Self {
        let data_local_dir = dirs::data_local_dir().expect("The OS doesn't provide a local data directory");
        let config_dir = dirs::config_dir().expect("The OS doesn't provide a config directory");
        let ata_data_dir = data_local_dir.join("ATA");

        Self {
            executable: executable_path(&data_local_dir),
            data_file: ata_data_dir.join("data.json"),
            settings_file: config_dir.join("ATA").join("settings.json"),
            uis_dir: ata_data_dir.join("UIs"),
            apps_dir: ata_data_dir.join("Apps"),
            downloads_dir: dirs::download_dir().or_else(dirs::home_dir).unwrap_or_default(),
        }
    }

    /// Every folder that must exist for ATA to work
    pub fn required_dirs(&self) -> Vec<&Path> {
        [
            self.executable.parent(),
            self.data_file.parent(),
            self.settings_file.parent(),
            Some(self.uis_dir.as_path()),
            Some(self.apps_dir.as_path()),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl Default for Paths {
    fn default() -> Self {
        Self::new()
    }
}

/// `%LOCALAPPDATA%\Programs\ATA\ATA.exe`
#[cfg(windows)]
fn executable_path(data_local_dir: &Path) -> PathBuf {
    data_local_dir.join("Programs").join("ATA").join("ATA.exe")
}

/// `~/.local/bin/ATA/ATA`
#[cfg(not(windows))]
fn executable_path(_data_local_dir: &Path) -> PathBuf {
    dirs::home_dir()
        .expect("The OS doesn't provide a home directory")
        .join(".local")
        .join("bin")
        .join("ATA")
        .join("ATA")
}

/// Global lazy-loaded [`Paths`] structure containing resolved application locations
pub static PATHS: LazyLock<Paths> = LazyLock::new(Paths::new);
