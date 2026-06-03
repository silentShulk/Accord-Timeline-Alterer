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

use std::fs::{create_dir_all, rename};

use std::path::PathBuf;

use thiserror::Error;

use crate::data::{Data, Mod, DataInteractionError};



/// Errors that could occur while enabling or disabling a mod
#[derive(Error, Debug)]
pub enum EnablingDisablingError {
    /// No mod with the given name was found in the data file
    #[error("No installed mod has the name {0}")]
    ModNotFound(String),

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

    /// The data file could not be updated after moving the files
    #[error("Couldn't update data file (data.json found inside data dir of OS). {0}")]
    DataSaving(#[from] DataInteractionError),
}



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
/// * [`EnablingDisablingError::ModNotFound`] if no mod with `mod_name` exists
/// * [`EnablingDisablingError::DotDotPath`] if a stored file path ends with `..`
/// * [`EnablingDisablingError::ParentlessOrEmptyPath`] if a stored file path has no parent
/// * [`EnablingDisablingError::Renaming`] if a file could not be moved
/// * [`EnablingDisablingError::DataSaving`] if the data file could not be updated
pub fn enable_mod(data: &mut Data, mod_name: String) -> Result<Mod, EnablingDisablingError>  {
	let Some(mod_to_enable) = data.get_mod_by_name(&mod_name) else {
		return Err(EnablingDisablingError::ModNotFound(mod_name))
	};
	let mut updated_files: Vec<PathBuf> = vec![];
	
    for file in &mod_to_enable.1.files {
    	let Some(filename) = file.file_name() else {
     		return Err(EnablingDisablingError::DotDotPath(file.to_path_buf()))
     	};
     	let Some(parent) = file.parent() else {
      		return Err(EnablingDisablingError::ParentlessOrEmptyPath(file.to_path_buf()))
      	};
      	let Some(enabled_folder) = parent.parent() else {
       		return Err(EnablingDisablingError::ParentlessOrEmptyPath(parent.to_path_buf()))
       	};
        
        let new_path = enabled_folder.join(filename);
      	rename(file, &new_path)?;
        updated_files.push(new_path);
    }

    data.switch_mod_state(mod_to_enable.0, updated_files)?;
    
    Ok(data.mods[mod_to_enable.0].clone())
}

/// Moves an enabled mod's files into a `.disabled/` subdirectory and marks it as disabled
///
/// For each file currently in the game's asset folder, this function creates a
/// `.disabled/` subdirectory alongside it (if it does not already exist), moves
/// the file there, and records the new path.  After all files are moved,
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
/// * [`EnablingDisablingError::ModNotFound`] if no mod with `mod_name` exists
/// * [`EnablingDisablingError::DotDotPath`] if a stored file path ends with `..`
/// * [`EnablingDisablingError::ParentlessOrEmptyPath`] if a stored file path has no parent
/// * [`EnablingDisablingError::FolderCreation`] if the `.disabled/` directory could not be created
/// * [`EnablingDisablingError::Renaming`] if a file could not be moved
/// * [`EnablingDisablingError::DataSaving`] if the data file could not be updated
pub fn disable_mod(data: &mut Data, mod_name: String) -> Result<Mod, EnablingDisablingError>  {
	let Some(mod_to_disable) = data.get_mod_by_name(&mod_name) else {
		return Err(EnablingDisablingError::ModNotFound(mod_name))
	};
	let mut updated_files: Vec<PathBuf> = vec![];
	
    for file in &mod_to_disable.1.files {
    	let Some(filename) = file.file_name() else {
     		return Err(EnablingDisablingError::DotDotPath(file.to_path_buf()))
     	};
     	let Some(parent) = file.parent() else {
      		return Err(EnablingDisablingError::ParentlessOrEmptyPath(file.to_path_buf()))
      	};
      	let disabled_folder = parent.join(".disabled/");
       
        create_dir_all(&disabled_folder)
            .map_err(|er| EnablingDisablingError::FolderCreation(disabled_folder.to_path_buf(), er))?;
       
      	rename(file, disabled_folder.join(filename))?;
       
        updated_files.push(disabled_folder.join(filename));
    }
    
    data.switch_mod_state(mod_to_disable.0, updated_files)?;
    
    Ok(data.mods[mod_to_disable.0].clone())
}
