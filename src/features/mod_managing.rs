//! **mod_managing** is a module that contains the functions needed to toggle mods on and off
//!
//! Enabling and disabling both work by physically moving the mod's files between two
//! locations on disk: the game's asset folder (enabled) and a `.disabled/` subdirectory
//! inside that folder (disabled). The new file paths are then persisted back to the
//! data file so ATA always knows where to find every mod.
//!
//! This includes:
//! * **listing**: Sorting tracked mods according to user preferences
//! * **enabling**: Moving mod files from `.disabled/` back into the game's asset folder
//! * **disabling**: Moving mod files out of the game's asset folder into `.disabled/`
//!
//! API functions: [`enable_mod`], [`disable_mod`], [`list_mods`]

use crate::utils::files::{get_filename_or_err, get_parent_or_err, FilesInteractionError};
use crate::data::mods::{Mods, DataInteractionError, Mod};
use crate::data::settings::SortingOrder;

use std::fs::{create_dir_all, rename};
use std::path::{Path, PathBuf};

use thiserror::Error;



/// Toggles a mod between enabled and disabled by moving its files and persisting the new state
///
/// If the mod is currently enabled, its files are moved into a `.disabled/` subdirectory
/// next to them (see [`disable_files`]). If it is currently disabled, they are moved back
/// one level up into the game's asset folder (see [`enable_files`]). After all files are
/// moved, [`Mods::switch_mod_state`] is called to flip the `enabled` flag and persist the
/// updated paths to the data file.
///
/// # Arguments
/// * `data` - Mutable reference to the current [`Mods`] state
/// * `mod_name` - Name of the mod to toggle
///
/// # Returns
/// * [`Ok`] -> A clone of the [`Mod`] after its state has been toggled
/// * [`Err`] -> The type of error that occurred
///
/// # Errors
/// * [`ModManagingError::DataSaving`] if no mod with `mod_name` exists, or if the data file could not be updated
/// * [`ModManagingError::FilesInteraction`] if a stored file path has no filename or no parent
/// * [`ModManagingError::FolderCreation`] if the `.disabled/` directory could not be created (disabling only)
/// * [`ModManagingError::Renaming`] if a file could not be moved
pub fn toggle_mod(mod_name: &str, data: &mut Mods) -> Result<Mod, ModManagingError> {
    let (index, mod_to_toggle) = data.get_mod_by_name(mod_name)?;

    let updated_files = toggle_files(mod_to_toggle.files, mod_to_toggle.enabled)?;

    data.switch_mod_state(index, updated_files)?;

    Ok(data.mods[index].clone())
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

    /// A path component of a stored file could not be extracted
    #[error("An error occurred while interacting with files. {0}")]
    FilesInteraction(#[from] FilesInteractionError),
}

/// Toggles files between the enabled folder and `.disabled/`
///
/// Files that are currently enabled are moved into `.disabled/`, and files that are
/// currently disabled are moved back out into the game's asset folder.
///
/// # Arguments
/// * `files` - Paths of the files to move
/// * `currently_enabled` - Current state of the files (`true` if they are in the game's asset folder)
///
/// # Returns
/// * [`Ok`] -> List of the files' new paths
/// * [`Err`] -> [`ModManagingError`] if path extraction, folder creation or renaming fails
fn toggle_files(
    files: impl IntoIterator<Item = PathBuf>,
    currently_enabled: bool,
) -> Result<Vec<PathBuf>, ModManagingError> {
    files
        .into_iter()
        .map(|file| {
            // `get_toggled_folder` takes the *target* state, so flip the current one
            let (filename, target_folder) = get_toggled_folder(!currently_enabled, &file)?;

            // No-op if the folder already exists (always the case when enabling)
            create_dir_all(&target_folder)
                .map_err(|er| ModManagingError::FolderCreation(target_folder.clone(), er))?;

            let new_path = target_folder.join(filename);
            rename(file, &new_path)?;

            Ok(new_path)
        })
        .collect()
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
    file: &'a Path,
) -> Result<(String, PathBuf), ModManagingError> {
    let filename = get_filename_or_err(file)?;
    
    if enabled {
        let enabled_folder = get_parent_or_err(&get_parent_or_err(file)?)?;

        Ok((filename, enabled_folder.to_path_buf()))
    } else {
        let disabled_folder = get_parent_or_err(file)?.join(".disabled/");

        Ok((filename, disabled_folder))
    }
}