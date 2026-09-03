//! **mod_managing** is a module that contains the functions needed to toggle mods on and off
//!
//! Enabling and disabling both work by physically moving the mod's files between two
//! locations on disk: the game's asset folder (enabled) and a `.disabled/` subdirectory
//! inside that folder (disabled). The new file paths are then persisted back to the
//! data file so ATA always knows where to find every mod.
//!
//! This includes:
//! * **enabling**: Moving mod files from `.disabled/` back into the game's asset folder
//! * **disabling**: Moving mod files out of the game's asset folder into `.disabled/`
//! * **listing**: Sorting tracked mods according to user preferences
//!
//! Main functions: [`enable_mod`], [`disable_mod`]

use crate::utils::files::{get_filename_or_err, get_parent_or_err, FilesInteractionError};
use crate::data::{Data, DataInteractionError, Mod};
use crate::settings::SortingOrder;

use std::fs::{create_dir_all, rename};
use std::path::PathBuf;

use thiserror::Error;



/// Moves a disabled mod's files back into the game's asset folder and marks it as enabled
///
/// For each file currently stored in a `.disabled/` subdirectory, this function
/// moves it one level up to the parent game asset folder and records the new path.
/// After all files are moved, [`Data::switch_mod_state`] is called to toggle the
/// `enabled` flag and persist the updated paths to the data file.
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
/// * [`ModManagingError::DataSaving`] if no mod with `mod_name` exists, or the data file could not be updated
/// * [`ModManagingError::AlreadyEnabled`] if the mod is already enabled
/// * [`ModManagingError::FilesInteraction`] if a stored file path has no name/parent
/// * [`ModManagingError::Renaming`] if a file could not be moved
pub fn enable_mod(data: &mut Data, mod_name: String) -> Result<Mod, ModManagingError> {
    let mod_to_enable = data.get_mod_by_name(&mod_name)?;
    if mod_to_enable.1.enabled {
        return Err(ModManagingError::AlreadyEnabled(mod_name));
    }

    let updated_files = toggle_files_state(mod_to_enable.1)?;

    data.switch_mod_state(mod_to_enable.0, updated_files)?;

    Ok(data.mods[mod_to_enable.0].clone())
}

/// Moves an enabled mod's files into a `.disabled/` subdirectory and marks it as disabled
///
/// For each file currently in the game's asset folder, this function creates a
/// `.disabled/` subdirectory alongside it (if it does not already exist), moves
/// the file there, and records the new path. After all files are moved,
/// [`Data::switch_mod_state`] is called to toggle the `enabled` flag and persist
/// the updated paths to the data file.
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
/// * [`ModManagingError::DataSaving`] if no mod with `mod_name` exists, or the data file could not be updated
/// * [`ModManagingError::AlreadyDisabled`] if the mod is already disabled
/// * [`ModManagingError::FilesInteraction`] if a stored file path has no name/parent
/// * [`ModManagingError::FolderCreation`] if the `.disabled/` directory could not be created
/// * [`ModManagingError::Renaming`] if a file could not be moved
pub fn disable_mod(data: &mut Data, mod_name: String) -> Result<Mod, ModManagingError> {
    let mod_to_disable = data.get_mod_by_name(&mod_name)?;
    if !mod_to_disable.1.enabled {
        return Err(ModManagingError::AlreadyDisabled(mod_name));
    }
    let updated_files = toggle_files_state(mod_to_disable.1)?;

    data.switch_mod_state(mod_to_disable.0, updated_files)?;

    Ok(data.mods[mod_to_disable.0].clone())
}

/// Errors that could occur while enabling or disabling a mod
#[derive(Error, Debug)]
pub enum ModManagingError {
    /// The `.disabled/` directory could not be created
    #[error("Couldn't create {0}. {1}")]
    FolderCreation(PathBuf, std::io::Error),

    /// A file could not be moved between the enabled and disabled locations
    #[error("Couldn't move file to enabled/disabled folder. {0}")]
    Renaming(#[from] std::io::Error),

    /// The data file could not be updated after moving the files
    #[error("Couldn't update data file (data.json found inside data dir of OS). {0}")]
    DataSaving(#[from] DataInteractionError),

    /// The requested mod is already enabled
    #[error("\"{0}\" is already enabled")]
    AlreadyEnabled(String),

    /// The requested mod is already disabled
    #[error("\"{0}\" is already disabled")]
    AlreadyDisabled(String),

    /// A path component of a stored file could not be extracted
    #[error("An error occurred while interacting with files. {0}")]
    FilesInteraction(#[from] FilesInteractionError),
}

/// Returns a copy of the mods vector sorted according to the requested [`SortingOrder`]
///
/// # Arguments
/// * `sorting_order` - Criterion used to sort the mod list
/// * `mods` - Reference to the slice of [`Mod`]s to sort
///
/// # Returns
/// * A [`Vec<Mod>`] sorted based on `sorting_order`
pub fn list_mods(sorting_order: &SortingOrder, mods: &[Mod]) -> Vec<Mod> {
    let mut sorted_mods: Vec<Mod> = mods.to_vec();

    match sorting_order {
        SortingOrder::ModType => sorted_mods.sort_unstable_by_key(|m| m.mod_type),
        SortingOrder::InstallDate => (),
        SortingOrder::EnableStatus => sorted_mods.sort_unstable_by_key(|m| m.enabled),
        SortingOrder::Alphabetical => sorted_mods.sort_unstable_by_key(|m| m.name.clone()),
        SortingOrder::Size => sorted_mods.sort_unstable_by_key(|m| m.files.len()),
    };

    sorted_mods
}

/// Dispatches a mod to be enabled or disabled depending on its current state
///
/// # Arguments
/// * `mod_to_enable` - The mod whose files should be toggled
///
/// # Returns
/// * [`Ok`] -> List of updated [`PathBuf`]s after moving
/// * [`Err`] -> [`EnablingDisablingError`] if file moving fails
fn toggle_files_state(mod_to_enable: Mod) -> Result<Vec<PathBuf>, ModManagingError> {
    if mod_to_enable.enabled {
        disable_files(mod_to_enable.files)
    } else {
        enable_files(mod_to_enable.files)
    }
}

/// Moves files from `.disabled/` subdirectories back up into their parent active folders
///
/// # Arguments
/// * `files_to_enable` - Paths of disabled files to enable
///
/// # Returns
/// * [`Ok`] -> List of updated active file paths
/// * [`Err`] -> [`EnablingDisablingError`] if renaming fails
fn enable_files(files_to_enable: Vec<PathBuf>) -> Result<Vec<PathBuf>, ModManagingError> {
    let mut updated_files: Vec<PathBuf> = vec![];

    for file in files_to_enable {
        let (filename, enabled_folder) = get_toggled_folder(true, &file)?;

        let new_path = enabled_folder.join(filename);
        rename(file, &new_path)?;

        updated_files.push(new_path);
    }

    Ok(updated_files)
}

/// Moves files from their active asset folders into adjacent `.disabled/` subdirectories
///
/// # Arguments
/// * `files_to_disable` - Paths of active files to disable
///
/// # Returns
/// * [`Ok`] -> List of updated disabled file paths
/// * [`Err`] -> [`EnablingDisablingError`] if creating directories or renaming fails
fn disable_files(files_to_disable: Vec<PathBuf>) -> Result<Vec<PathBuf>, ModManagingError> {
    let mut updated_files: Vec<PathBuf> = vec![];

    for file in files_to_disable {
        let (filename, disabled_folder) = get_toggled_folder(false, &file)?;

        create_dir_all(&disabled_folder).map_err(|er| {
            ModManagingError::FolderCreation(disabled_folder.to_path_buf(), er)
        })?;

        let new_path = disabled_folder.join(filename);
        rename(file, &new_path)?;

        updated_files.push(new_path);
    }

    Ok(updated_files)
}

/// Determines the target directory and filename for enabling or disabling a file
///
/// # Arguments
/// * `enabled` - `true` if target location is active folder (enabling), `false` for `.disabled/` (disabling)
/// * `file` - Original path of the file
///
/// # Returns
/// * [`Ok`]`((&str, PathBuf))` -> Tuple containing the filename and target directory path
/// * [`Err`] -> [`ModManagingError`] if path component extraction fails
fn get_toggled_folder<'a>(
    enabled: bool,
    file: &'a PathBuf,
) -> Result<(&'a str, PathBuf), ModManagingError> {
    let filename = get_filename_or_err(file)?;
    
    if enabled {
        let enabled_folder = get_parent_or_err(&get_parent_or_err(file)?)?;

        Ok((filename, enabled_folder.to_path_buf()))
    } else {
        let disabled_folder = get_parent_or_err(file)?.join(".disabled/");

        Ok((filename, disabled_folder))
    }
}