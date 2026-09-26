//! **mod_managing** is a module that contains the functions needed to toggle mods on and off
//!
//! Enabling and disabling both work by physically moving the mod's files between two
//! locations on disk: the game's asset folder (enabled) and a `.disabled/` subdirectory
//! inside that folder (disabled). The new file paths are then persisted back to the
//! data file so ATA always knows where to find every mod.
//!
//! If the mod replaced original game files, those are put back in place while the
//! mod is disabled, and set aside again when it is re-enabled.
//! Files inside `reshade-shaders/` are never moved (see [`Mod::is_static_file`]).
//!
//! Every change is transactional: if one file can't be moved, nothing changes.
//!
//! Main functions: [`enable_mod`], [`disable_mod`]

use crate::data::mods::{Backup, DataInteractionError, Mod, Mods, same_path};
use crate::features::backup_location;
use crate::features::transaction::Transaction;
use crate::utils::files::unique_path;

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Moves a disabled mod's files back into the game's asset folders and marks it as enabled
///
/// # Arguments
/// * `data` - Mutable reference to the current [`Mods`] state
/// * `mod_name` - Name of the mod to enable
/// * `game_path` - The game folder (empty folders left inside it are cleaned up)
///
/// # Returns
/// * [`Ok`] -> A clone of the now-enabled [`Mod`]
/// * [`Err`] -> The type of error that occurred (nothing was changed)
///
/// # Errors
/// * [`ModManagingError::DataSaving`] if no mod with `mod_name` exists, or the data file could not be updated
/// * [`ModManagingError::AlreadyEnabled`] if the mod is already enabled
/// * [`ModManagingError::Moving`] if a file could not be moved
pub fn enable_mod(data: &mut Mods, mod_name: &str, game_path: &Path) -> Result<Mod, ModManagingError> {
    set_mod_state(data, mod_name, true, game_path)
}

/// Moves an enabled mod's files into `.disabled/` subdirectories and marks it as disabled
///
/// # Arguments
/// * `data` - Mutable reference to the current [`Mods`] state
/// * `mod_name` - Name of the mod to disable
/// * `game_path` - The game folder (empty folders left inside it are cleaned up)
///
/// # Returns
/// * [`Ok`] -> A clone of the now-disabled [`Mod`]
/// * [`Err`] -> The type of error that occurred (nothing was changed)
///
/// # Errors
/// * [`ModManagingError::DataSaving`] if no mod with `mod_name` exists, or the data file could not be updated
/// * [`ModManagingError::AlreadyDisabled`] if the mod is already disabled
/// * [`ModManagingError::Moving`] if a file could not be moved
pub fn disable_mod(data: &mut Mods, mod_name: &str, game_path: &Path) -> Result<Mod, ModManagingError> {
    set_mod_state(data, mod_name, false, game_path)
}

/// Errors that could occur while enabling or disabling a mod
#[derive(Error, Debug)]
pub enum ModManagingError {
    /// A file could not be moved between the enabled and disabled locations
    #[error("Couldn't move '{}' to '{}'. {source}", from.display(), to.display())]
    Moving {
        from: PathBuf,
        to: PathBuf,
        source: std::io::Error,
    },

    /// The data file could not be read or updated
    #[error("{0}")]
    DataSaving(#[from] DataInteractionError),

    /// The requested mod is already enabled
    #[error("'{0}' is already enabled")]
    AlreadyEnabled(String),

    /// The requested mod is already disabled
    #[error("'{0}' is already disabled")]
    AlreadyDisabled(String),
}

/// Enables (`enable == true`) or disables a mod, then saves the data file
fn set_mod_state(data: &mut Mods, mod_name: &str, enable: bool, game_path: &Path) -> Result<Mod, ModManagingError> {
    let index = data.index_of(mod_name)?;
    let current = &data.mods[index];

    match (current.enabled, enable) {
        (true, true) => return Err(ModManagingError::AlreadyEnabled(current.name.clone())),
        (false, false) => return Err(ModManagingError::AlreadyDisabled(current.name.clone())),
        _ => {}
    }

    let mut updated = current.clone();
    let mut tx = Transaction::new(game_path);

    if enable {
        enable_files(&mut updated, &mut tx, game_path)?;
    } else {
        disable_files(&mut updated, &mut tx)?;
    }
    updated.enabled = enable;

    let mut new_data = data.clone();
    new_data.mods[index] = updated.clone();
    new_data.save()?;

    tx.commit();
    *data = new_data;

    Ok(updated)
}

/// Moves every movable file of `the_mod` from `.disabled/` back to its enabled location,
/// setting aside the original game file that sits there (if any)
fn enable_files(the_mod: &mut Mod, tx: &mut Transaction, game_path: &Path) -> Result<(), ModManagingError> {
    let mut new_backups: Vec<Backup> = Vec::new();

    for file in the_mod.files.iter_mut().filter(|f| !Mod::is_static_file(f)) {
        let active = Mod::active_path(file);
        if *file == active {
            continue;
        }

        // Whatever occupies the enabled location is the original file (restored while
        // the mod was disabled) or a file placed there by hand: keep it safe either way
        if active.exists() {
            let backup = match the_mod.backups.iter().position(|b| same_path(&b.original, &active)) {
                Some(i) => {
                    let backup = unique_path(&the_mod.backups[i].backup);
                    the_mod.backups[i].backup = backup.clone();
                    backup
                }
                None => {
                    let backup = unique_path(&backup_location(&active, game_path));
                    new_backups.push(Backup { original: active.clone(), backup: backup.clone() });
                    backup
                }
            };
            move_file(tx, &active, &backup)?;
        }

        move_file(tx, file, &active)?;
        *file = active;
    }

    the_mod.backups.extend(new_backups);
    Ok(())
}

/// Moves every movable file of `the_mod` to its `.disabled/` location,
/// putting back the original game file it replaced (if any)
fn disable_files(the_mod: &mut Mod, tx: &mut Transaction) -> Result<(), ModManagingError> {
    for file in the_mod.files.iter_mut().filter(|f| !Mod::is_static_file(f)) {
        let active = Mod::active_path(file);
        let disabled = Mod::disabled_path(file);
        if *file == disabled {
            continue;
        }

        move_file(tx, file, &disabled)?;
        *file = disabled;

        let backup = the_mod.backups.iter().find(|b| same_path(&b.original, &active));
        if let Some(backup) = backup.filter(|b| b.backup.exists()) {
            move_file(tx, &backup.backup, &active)?;
        }
    }

    Ok(())
}

/// [`Transaction::move_file`] with a descriptive error
fn move_file(tx: &mut Transaction, from: &Path, to: &Path) -> Result<(), ModManagingError> {
    tx.move_file(from, to).map_err(|source| ModManagingError::Moving {
        from: from.to_path_buf(),
        to: to.to_path_buf(),
        source,
    })
}
