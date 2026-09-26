//! **wipe** is a module that removes every installed mod at once
//!
//! Main function: [`uninstall_all`]

use crate::data::mods::Mods;
use crate::features::uninstallation::uninstall_mod;

use std::path::Path;

use serde::Serialize;

/// Outcome of [`uninstall_all`]
#[derive(Serialize, Debug, Default)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct WipeReport {
    /// Names of the mods that were uninstalled
    pub uninstalled: Vec<String>,
    /// Mods that couldn't be uninstalled, with the reason
    pub failed: Vec<WipeFailure>,
}

/// A mod [`uninstall_all`] couldn't uninstall
#[derive(Serialize, Debug)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct WipeFailure {
    pub name: String,
    pub error: String,
}

/// Uninstalls every mod, newest first, restoring the original game files
///
/// A mod that fails to uninstall doesn't stop the others; it is reported in [`WipeReport::failed`].
pub fn uninstall_all(data: &mut Mods, game_path: &Path) -> WipeReport {
    let names: Vec<String> = data.mods.iter().rev().map(|m| m.name.clone()).collect();
    let mut report = WipeReport::default();

    for name in names {
        match uninstall_mod(data, &name, game_path) {
            Ok(_) => report.uninstalled.push(name),
            Err(err) => report.failed.push(WipeFailure { name, error: err.to_string() }),
        }
    }

    report
}
