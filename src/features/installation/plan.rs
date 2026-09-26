//! **plan** decides, before touching anything, what installing the analysed files implies
//!
//! For every file it computes the absolute destination and checks it against:
//! * the other files of the same mod (two files can't go to the same place)
//! * the files of the installed mods (conflicts). A file already installed by another mod
//!   with identical content inside `reshade-shaders/` is simply shared, not a conflict
//!
//! Main function: [`plan_installation`]

use super::InstallationError;
use super::analysis::ModFile;
use crate::data::mods::{Mod, Mods};
use crate::utils::files::files_equal;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// A file of the new mod that would replace a file of an installed mod
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Conflict {
    /// The contested file (its enabled location in the game folder)
    pub file: PathBuf,
    /// Name of the installed mod currently owning it
    pub owner: String,
}

/// What happens to a single file during the installation
pub(super) enum Placement {
    /// Nothing is at the destination
    Free,
    /// The destination belongs to an installed mod (index into [`Mods::mods`]),
    /// its file will be taken over (only allowed when overwriting)
    TakeOver(usize),
    /// The destination already contains the very same file, installed by another mod:
    /// the file is shared and nothing is copied
    Shared,
}

/// A file ready to be installed
pub(super) struct PlannedFile {
    pub source: PathBuf,
    /// Absolute destination
    pub destination: PathBuf,
    pub placement: Placement,
}

/// Everything the installation will do
pub(super) struct InstallPlan {
    pub files: Vec<PlannedFile>,
    pub conflicts: Vec<Conflict>,
}

/// Computes where every file goes and what it conflicts with
///
/// # Errors
/// * [`InstallationError::AmbiguousFiles`] if several files of the mod would be installed at the same place
pub(super) fn plan_installation(files: &[ModFile], game_path: &Path, data: &Mods) -> Result<InstallPlan, InstallationError> {
    check_duplicates(files)?;

    let mut planned = Vec::with_capacity(files.len());
    let mut conflicts = Vec::new();

    for file in files {
        let destination = game_path.join(&file.destination);

        let placement = match data.owner_of(&destination, None) {
            None => Placement::Free,
            Some(_) if is_identical_shared_file(&file.source, &destination) => Placement::Shared,
            Some(owner) => {
                conflicts.push(Conflict {
                    file: destination.clone(),
                    owner: data.mods[owner].name.clone(),
                });
                Placement::TakeOver(owner)
            }
        };

        planned.push(PlannedFile {
            source: file.source.clone(),
            destination,
            placement,
        });
    }

    Ok(InstallPlan { files: planned, conflicts })
}

/// Errors if two files of the mod would be installed at the same place
/// (typically an archive containing alternative versions of the same mod)
fn check_duplicates(files: &[ModFile]) -> Result<(), InstallationError> {
    let mut by_destination: HashMap<String, Vec<&ModFile>> = HashMap::new();
    for file in files {
        // Case-insensitive: the game (and Windows) don't tell `A.dds` and `a.dds` apart
        let key = file.destination.to_string_lossy().replace('\\', "/").to_lowercase();
        by_destination.entry(key).or_default().push(file);
    }

    let mut duplicates: Vec<PathBuf> = by_destination
        .into_values()
        .filter(|same_place| same_place.len() > 1)
        .flatten()
        .map(|f| f.source.clone())
        .collect();

    if duplicates.is_empty() {
        Ok(())
    } else {
        duplicates.sort();
        Err(InstallationError::AmbiguousFiles(duplicates))
    }
}

/// Whether `source` can share the already installed `destination` instead of replacing it
fn is_identical_shared_file(source: &Path, destination: &Path) -> bool {
    Mod::is_static_file(destination) && files_equal(source, destination).unwrap_or(false)
}
