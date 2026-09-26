//! **uninstallation** is a module that contains the functions needed to uninstall a mod
//!
//! This includes:
//! * **locating**: Finding the mod by name in the data file
//! * **deleting**: Removing every file that belongs to the mod from disk
//!   (files still used by another mod, like shared ReShade shaders, are kept)
//! * **restoring**: Putting back the original game files the mod replaced
//! * **updating the saved data**: Removing the mod entry from the data file
//!
//! Uninstalling is transactional: if a file can't be removed, nothing changes.
//! Files that were already missing are not an error.
//!
//! Main function: [`uninstall_mod`]

use crate::data::mods::{DataInteractionError, Mod, Mods};
use crate::features::transaction::Transaction;

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Removes a mod's files from disk, restores the files it replaced and deletes its entry from the data file
///
/// # Arguments
/// * `data` - Mutable reference to the current [`Mods`] state
/// * `mod_name` - Name of the mod to uninstall
/// * `game_path` - The game folder (empty folders left inside it are cleaned up)
///
/// # Returns
/// * [`Ok`] -> The [`Mod`] that was uninstalled
/// * [`Err`] -> The type of error that occurred (nothing was changed)
///
/// # Errors
/// * [`UninstallationError::DataSaving`] if no mod with `mod_name` exists, or the data file could not be updated
/// * [`UninstallationError::FileDeletion`] if one of the mod's files could not be removed from disk
/// * [`UninstallationError::Restoring`] if an original game file could not be restored
pub fn uninstall_mod(data: &mut Mods, mod_name: &str, game_path: &Path) -> Result<Mod, UninstallationError> {
    let index = data.index_of(mod_name)?;

    let mut new_data = data.clone();
    let removed = new_data.mods.remove(index);
    let mut tx = Transaction::new(game_path);

    for file in &removed.files {
        let active = Mod::active_path(file);

        // Shared files (e.g. identical ReShade shaders) stay while another mod uses them
        if let Some(other) = new_data.owner_of(&active, None) {
            if let Some(backup) = removed.backup_for(&active) {
                new_data.mods[other].backups.push(backup.clone());
            }
            continue;
        }

        tx.discard_file(file)
            .map_err(|source| UninstallationError::FileDeletion { file: file.clone(), source })?;

        // While a file is in `.disabled/`, the original it replaced is already back in place
        let is_in_place = *file == active;
        if let Some(backup) = removed.backup_for(&active).filter(|b| is_in_place && b.backup.exists()) {
            tx.move_file(&backup.backup, &active)
                .map_err(|source| UninstallationError::Restoring { file: active.clone(), source })?;
        }
    }

    new_data.save()?;
    tx.commit();
    *data = new_data;

    Ok(removed)
}

/// Errors that could occur while uninstalling a mod
#[derive(Error, Debug)]
pub enum UninstallationError {
    /// A specific file could not be removed from the game directory
    #[error("Couldn't remove '{}' from the game's directory. {source}", file.display())]
    FileDeletion { file: PathBuf, source: std::io::Error },

    /// An original game file could not be put back
    #[error("Couldn't restore the original '{}'. {source}", file.display())]
    Restoring { file: PathBuf, source: std::io::Error },

    /// The data file could not be read or updated
    #[error("{0}")]
    DataSaving(#[from] DataInteractionError),
}
