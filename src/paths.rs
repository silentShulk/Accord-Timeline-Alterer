//! **paths** is a module that contains application-wide path definitions
//!
//! Evaluates OS-specific paths lazily at runtime for configuration, binaries, data, UIs, and app assets.
//!
//! Main static: [`PATHS`]

use dirs::{config_dir, data_local_dir, home_dir};

use std::path::PathBuf;
use std::sync::LazyLock;

use serde::Serialize;

/// Container struct for all paths used by ATA across supported operating systems
#[derive(Serialize)]
pub struct Paths {
    /// Path to the primary executable binary
    pub executable: PathBuf,
    /// Path to `data.json` storing runtime mod state
    pub data_file: PathBuf,
    /// Path to `settings.json` storing user configurations
    pub settings_file: PathBuf,
    /// Path to user interface template/asset directory
    pub uis_dir: PathBuf,
    /// Path to application dependencies directory
    pub apps_dir: PathBuf,
}

/// Global lazy-loaded [`Paths`] structure containing resolved application locations
pub static PATHS: LazyLock<Paths> = LazyLock::new(|| {
    Paths {
        #[cfg(target_os = "linux")]
        executable: home_dir()
            .unwrap()
            .join(".local")
            .join("bin")
            .join("ATA")
            .join("ATA"),
        #[cfg(target_os = "windows")]
        executable: data_local_dir()
            .unwrap()
            .join("Programs")
            .join("ATA")
            .join("ATA.exe"),
        data_file: data_local_dir().unwrap().join("ATA").join("data.json"),
        settings_file: config_dir().unwrap().join("ATA").join("settings.json"),
        uis_dir: data_local_dir().unwrap().join("ATA").join("UIs"),
        apps_dir: data_local_dir().unwrap().join("ATA").join("Apps"),
    }
});