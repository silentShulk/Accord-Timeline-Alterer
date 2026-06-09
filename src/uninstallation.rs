//! **uninstallation** is a module that contains the functions needed to uninstall a mod
//!
//! This includes:
//! * **locating**: Finding the mod by name in the data file
//! * **deleting**: Removing every file that belongs to the mod from disk
//! * **updating the saved data**: Removing the mod entry from the data file
//!
//! Main function: [`uninstall_mod`]

use crate::data::{Data, DataInteractionError, Mod};

use std::fs::remove_file;

use std::path::PathBuf;

use thiserror::Error;



/// Removes a mod's files from disk and deletes its entry from the data file
///
/// # Arguments
/// * `config` - Mutable reference to the current [`Data`] state
/// * `mod_name` - Name of the mod to uninstall
///
/// # Returns
/// * [`Ok`] -> A clone of the [`Mod`] that was uninstalled
/// * [`Err`] -> The type of error that occurred
///
/// # Errors
/// * [`UninstallationError::ModNotFound`] if no mod with `mod_name` exists in the data file
/// * [`UninstallationError::FileDeletion`] if one of the mod's files could not be removed from disk
/// * [`UninstallationError::DataSaving`] if the data file could not be updated after deletion
pub fn uninstall_mod(config: &mut Data, mod_name: String) -> Result<Mod, UninstallationError> {
    let Some(mod_to_uninstall) = config.get_mod_by_name(&mod_name) else {
        return Err(UninstallationError::ModNotFound(mod_name));
    };

    remove_mod_files(&mod_to_uninstall.1.files)?;

    config
        .remove_mod(mod_to_uninstall.0)
        .map_err(|e| UninstallationError::DataSaving(e))?;

    Ok(mod_to_uninstall.1)
}



/// Errors that could occur while uninstalling a mod
#[derive(Error, Debug)]
pub enum UninstallationError {
    /// A file belonging to the mod could not be accessed or deleted
    #[error("Encountered an error while trying to access a mod file. {0}")]
    FileAccessing(#[from] std::io::Error),

    /// No mod with the requested name was found in the data file
    #[error("No mod named '{0}' found")]
    ModNotFound(String),

    /// A specific file could not be removed from the game directory
    #[error("Couldn't remove {0} from the game's directory. {1}")]
    FileDeletion(PathBuf, std::io::Error),

    /// The data file could not be updated after the mod files were deleted
    #[error("Couldn't update data file (data.json found inside data dir of OS). {0}")]
    DataSaving(#[from] DataInteractionError),
}



/// Deletes every file in `mod_files` from disk
///
/// Stops at the first failure and returns an error; files deleted before the
/// failure are not restored.
///
/// # Arguments
/// * `mod_files` - Slice of absolute paths to the files to delete
///
/// # Returns
/// * [`Ok`] -> `()` if all files were deleted successfully
/// * [`Err`] -> The type of error that occurred
///
/// # Errors
/// * [`UninstallationError::FileDeletion`] if any file could not be removed
pub fn remove_mod_files(mod_files: &[PathBuf]) -> Result<(), UninstallationError> {
    for file in mod_files {
        remove_file(&file).map_err(|er| UninstallationError::FileDeletion(file.clone(), er))?;
    }

    Ok(())
}
