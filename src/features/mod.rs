//! **features** contains everything ATA can do
//!
//! * [`installation`]: installing a mod from an archive or a folder (`archives` feature)
//! * [`uninstallation`]: removing a mod and restoring what it replaced
//! * [`mod_managing`]: enabling/disabling mods
//! * [`listing`]: sorting mods for display
//! * [`game`]: launching, validating and detecting the game
//! * [`wipe`]: uninstalling every mod at once

pub mod game;
#[cfg(feature = "archives")]
pub mod installation;
pub mod listing;
pub mod mod_managing;
pub mod uninstallation;
pub mod wipe;

mod transaction;

use std::path::{Path, PathBuf};

/// Folder of the game in which ATA keeps the original files replaced by mods
pub const BACKUP_DIR_NAME: &str = ".ata-backup";

/// Where the original game file at `original` is kept while a mod replaces it
///
/// `<game>/data/movie/ev0010.usm` -> `<game>/.ata-backup/data/movie/ev0010.usm`
/// (same drive as the game, so setting it aside is an instant rename even for big files)
pub(crate) fn backup_location(original: &Path, game_root: &Path) -> PathBuf {
    match original.strip_prefix(game_root) {
        Ok(relative) if !game_root.as_os_str().is_empty() => game_root.join(BACKUP_DIR_NAME).join(relative),
        _ => {
            let parent = original.parent().unwrap_or(Path::new(""));
            parent.join(BACKUP_DIR_NAME).join(original.file_name().unwrap_or_default())
        }
    }
}
