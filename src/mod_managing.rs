//! **mod_managing** is a module that contains the functions needed to toggle mods on and off
//!
//! Enabling and disabling both work by physically moving the mod's files between two
//! locations on disk: the game's asset folder (enabled) and a `.disabled/` subdirectory
//! inside that folder (disabled).  The new file paths are then persisted back to the
//! data file so ATA always knows where to find every mod.
//!
//! This includes:
//! * **enabling**: Moving mod files from `.disabled/` back into the game's asset folder
//! * **disabling**: Moving mod files out of the game's asset folder into `.disabled/`
//!
//! Main functions: [`enable_mod`], [`disable_mod`]
//!
//! Private helpers: [`build_enable_pairs`], [`build_disable_pairs`], [`execute_moves_with_rollback`]

use std::fs::{create_dir_all, rename};

use std::path::PathBuf;

use thiserror::Error;

use crate::data::{Data, Mod, DataInteractionError};

use crate::settings::SortingOrder;



/// Errors that could occur while enabling or disabling a mod
#[derive(Error, Debug)]
pub enum EnablingDisablingError {
    /// No mod with the given name was found in the data file
    #[error("No installed mod has the name {0}")]
    ModNotFound(String),

    /// The mod is already enabled
    #[error("The mod '{0}' is already enabled")]
    AlreadyEnabled(String),

    /// The mod is already disabled
    #[error("The mod '{0}' is already disabled")]
    AlreadyDisabled(String),

    /// A stored file path ends with `..`, making it impossible to extract a filename
    #[error("{0} ends with ..")]
    DotDotPath(PathBuf),

    /// A stored file path has no parent component (is root or empty)
    #[error("{0} is either root or an empty path")]
    ParentlessOrEmptyPath(PathBuf),

    /// The `.disabled/` directory could not be created
    #[error("Couldn't create {0}. {1}")]
    FolderCreation(PathBuf, std::io::Error),

    /// A file could not be moved between the enabled and disabled locations
    #[error("Couldn't move file from downloaded folder to game folder. {0}")]
    Renaming(#[from] std::io::Error),

    /// A file could not be moved back during rollback after a partial failure
    #[error("Couldn't rollback move for {0}: {1}")]
    RollbackFailed(PathBuf, std::io::Error),

    /// The data file could not be updated after moving the files
    #[error("Couldn't update data file (data.json found inside data dir of OS). {0}")]
    DataSaving(#[from] DataInteractionError),
}



pub fn list_mods(sorting_order: &SortingOrder, mods: &[Mod]) -> Vec<Mod> {
    let mut sorted_mods: Vec<Mod> = mods.into();

    match sorting_order {
        SortingOrder::ModType     => sorted_mods.sort_unstable_by_key(|m| m.mod_type),
        SortingOrder::InstallDate => (),
        SortingOrder::EnableStatus => sorted_mods.sort_unstable_by_key(|m| m.enabled),
        SortingOrder::Alphabetical => sorted_mods.sort_unstable_by_key(|m| m.name.clone()),
        SortingOrder::Size        => sorted_mods.sort_unstable_by_key(|m| m.files.len()),
    };

    sorted_mods
}

/// Moves a disabled mod's files back into the game's asset folder and marks it as enabled
///
/// # Arguments
/// * `data` - Mutable reference to the current [`Data`] state
/// * `mod_name` - Name of the mod to enable
///
/// # Returns
/// * [`Ok`] -> A clone of the now-enabled [`Mod`]
/// * [`Err`] -> The type of error that occurred
///
/// # Errors
/// * [`EnablingDisablingError::ModNotFound`] if no mod with `mod_name` exists
/// * [`EnablingDisablingError::AlreadyEnabled`] if the mod is already enabled
/// * [`EnablingDisablingError::DotDotPath`] if a stored file path ends with `..`
/// * [`EnablingDisablingError::ParentlessOrEmptyPath`] if a stored file path has no parent
/// * [`EnablingDisablingError::Renaming`] if a file could not be moved
/// * [`EnablingDisablingError::RollbackFailed`] if rollback itself failed
/// * [`EnablingDisablingError::DataSaving`] if the data file could not be updated
pub fn enable_mod(data: &mut Data, mod_name: String) -> Result<Mod, EnablingDisablingError> {
    let Some(mod_to_enable) = data.get_mod_by_name(&mod_name) else {
        return Err(EnablingDisablingError::ModNotFound(mod_name));
    };

    if mod_to_enable.1.enabled {
        return Err(EnablingDisablingError::AlreadyEnabled(mod_name));
    }

    let pairs        = build_enable_pairs(&mod_to_enable.1.files)?;
    let updated_files = execute_moves_with_rollback(pairs)?;

    data.switch_mod_state(mod_to_enable.0, updated_files)?;

    Ok(data.mods[mod_to_enable.0].clone())
}

/// Moves an enabled mod's files into a `.disabled/` subdirectory and marks it as disabled
///
/// # Arguments
/// * `data` - Mutable reference to the current [`Data`] state
/// * `mod_name` - Name of the mod to disable
///
/// # Returns
/// * [`Ok`] -> A clone of the now-disabled [`Mod`]
/// * [`Err`] -> The type of error that occurred
///
/// # Errors
/// * [`EnablingDisablingError::ModNotFound`] if no mod with `mod_name` exists
/// * [`EnablingDisablingError::AlreadyDisabled`] if the mod is already disabled
/// * [`EnablingDisablingError::DotDotPath`] if a stored file path ends with `..`
/// * [`EnablingDisablingError::ParentlessOrEmptyPath`] if a stored file path has no parent
/// * [`EnablingDisablingError::FolderCreation`] if the `.disabled/` directory could not be created
/// * [`EnablingDisablingError::Renaming`] if a file could not be moved
/// * [`EnablingDisablingError::RollbackFailed`] if rollback itself failed
/// * [`EnablingDisablingError::DataSaving`] if the data file could not be updated
pub fn disable_mod(data: &mut Data, mod_name: String) -> Result<Mod, EnablingDisablingError> {
    let Some(mod_to_disable) = data.get_mod_by_name(&mod_name) else {
        return Err(EnablingDisablingError::ModNotFound(mod_name));
    };

    if !mod_to_disable.1.enabled {
        return Err(EnablingDisablingError::AlreadyDisabled(mod_name));
    }

    let pairs         = build_disable_pairs(&mod_to_disable.1.files)?;
    let updated_files = execute_moves_with_rollback(pairs)?;

    data.switch_mod_state(mod_to_disable.0, updated_files)?;

    Ok(data.mods[mod_to_disable.0].clone())
}



/// Builds `(src, dst)` move pairs for re-enabling a mod
///
/// Each `src` is a file currently inside `.disabled/`; the matching `dst` is one
/// level up, back in the game's asset folder.  No files are moved by this function.
///
/// # Errors
/// * [`EnablingDisablingError::DotDotPath`] if a path ends with `..`
/// * [`EnablingDisablingError::ParentlessOrEmptyPath`] if a path has no parent
fn build_enable_pairs(files: &[PathBuf]) -> Result<Vec<(PathBuf, PathBuf)>, EnablingDisablingError> {
    let mut pairs = Vec::with_capacity(files.len());

    for file in files {
        let Some(filename) = file.file_name() else {
            return Err(EnablingDisablingError::DotDotPath(file.to_path_buf()));
        };
        let Some(disabled_folder) = file.parent() else {
            return Err(EnablingDisablingError::ParentlessOrEmptyPath(file.to_path_buf()));
        };
        let Some(enabled_folder) = disabled_folder.parent() else {
            return Err(EnablingDisablingError::ParentlessOrEmptyPath(disabled_folder.to_path_buf()));
        };

        pairs.push((file.to_path_buf(), enabled_folder.join(filename)));
    }

    Ok(pairs)
}

/// Builds `(src, dst)` move pairs for disabling a mod and creates the `.disabled/` dirs
///
/// Each `src` is a file currently in the game's asset folder; the matching `dst` is
/// inside a `.disabled/` subdirectory next to it.  The `.disabled/` dirs are created
/// here so that pair-building and dir-creation failures are caught before any file
/// is moved.  No files are moved by this function.
///
/// # Errors
/// * [`EnablingDisablingError::DotDotPath`] if a path ends with `..`
/// * [`EnablingDisablingError::ParentlessOrEmptyPath`] if a path has no parent
/// * [`EnablingDisablingError::FolderCreation`] if a `.disabled/` dir could not be created
fn build_disable_pairs(files: &[PathBuf]) -> Result<Vec<(PathBuf, PathBuf)>, EnablingDisablingError> {
    let mut pairs = Vec::with_capacity(files.len());

    for file in files {
        let Some(filename) = file.file_name() else {
            return Err(EnablingDisablingError::DotDotPath(file.to_path_buf()));
        };
        let Some(parent) = file.parent() else {
            return Err(EnablingDisablingError::ParentlessOrEmptyPath(file.to_path_buf()));
        };
        let disabled_folder = parent.join(".disabled/");

        create_dir_all(&disabled_folder)
            .map_err(|er| EnablingDisablingError::FolderCreation(disabled_folder.clone(), er))?;

        pairs.push((file.to_path_buf(), disabled_folder.join(filename)));
    }

    Ok(pairs)
}

/// Executes a list of `(src, dst)` file moves, rolling back on failure
///
/// Moves are performed in order.  If any `rename` fails, all previously completed
/// moves are reversed in reverse order (`dst → src`) before returning the error.
///
/// # Returns
/// * [`Ok`] -> `Vec<PathBuf>` of all destination paths (the new file locations)
/// * [`Err`] -> The error that caused the failure (rollback already performed)
///
/// # Errors
/// * [`EnablingDisablingError::Renaming`] if a move fails
/// * [`EnablingDisablingError::RollbackFailed`] if a rollback move itself fails
fn execute_moves_with_rollback(pairs: Vec<(PathBuf, PathBuf)>) -> Result<Vec<PathBuf>, EnablingDisablingError> {
    let mut moved: Vec<(PathBuf, PathBuf)> = Vec::with_capacity(pairs.len());

    for (src, dst) in &pairs {
        if let Err(err) = rename(src, dst) {
            for (orig_src, orig_dst) in moved.iter().rev() {
                rename(orig_dst, orig_src)
                    .map_err(|rb_err| EnablingDisablingError::RollbackFailed(orig_dst.clone(), rb_err))?;
            }
            return Err(EnablingDisablingError::Renaming(err));
        }
        moved.push((src.clone(), dst.clone()));
    }

    Ok(pairs.into_iter().map(|(_, dst)| dst).collect())
}



