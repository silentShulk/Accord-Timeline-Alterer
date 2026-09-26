//! **installation** is a module that contains the functions needed to install a mod
//!
//! Installing goes through these steps (reported to the caller through [`InstallStep`]):
//! 1. **checking**: the name, the source and the game folder are validated
//! 2. **extracting** ([`archive`]): the archive is extracted (a folder is used as is)
//! 3. **analysing** ([`analysis`], [`reshade`]): the mod type and each file's destination are found
//! 4. **checking conflicts** ([`plan`]): files owned by installed mods are detected
//! 5. **installing**: files are copied into the game; original game files that get
//!    replaced are set aside so they can be restored on uninstall
//! 6. **saving data**: the new mod is added to the data file
//!
//! Nothing is changed if any step fails: files already copied are removed and replaced
//! files are put back.
//!
//! Main function: [`install_mod`]

mod analysis;
mod archive;
mod plan;
mod reshade;

pub use archive::SUPPORTED_ARCHIVES;
pub use plan::Conflict;

use crate::data::mod_type::ModType;
use crate::data::mods::{Backup, DataInteractionError, Mod, Mods, same_path};
use crate::data::settings::{ConflictResolution, Settings};
use crate::features::backup_location;
use crate::features::game::{GamePathError, is_reshade_installed, validate_game_path};
use crate::features::transaction::Transaction;
use crate::utils::files::{FilesInteractionError, is_valid_folder_name, unique_path};
use plan::{InstallPlan, Placement};

use std::fmt;
use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

/// The steps of an installation, reported as they start
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum InstallStep {
    Checking,
    Extracting,
    Analysing,
    CheckingConflicts,
    Installing,
    SavingData,
    /// Not a step: something the user should know about (e.g. skipped files)
    Warning,
}

/// Installs a mod from an archive (or an extracted folder) and saves it in data with user-decided name
///
/// # Arguments
/// * `source` - Path to the archive (`.zip`, `.7z`, `.rar`) or folder containing the mod
/// * `name` - Name to assign to the mod
/// * `forced_overwrite` - Whether to overwrite the files of installed mods even when settings say to warn
/// * `settings` - Runtime application settings
/// * `data` - Mutable reference to the current [`Mods`] state
/// * `progress` - Called when each [`InstallStep`] starts, with a human readable message
///
/// # Returns
/// * [`Ok`] -> The installed [`Mod`]
/// * [`Err`] -> The type of error that occurred (the game folder and data are left untouched)
///
/// # Errors
/// * [`InstallationError::InvalidName`] if the name can't be used as a folder name
/// * [`InstallationError::NameAlreadyExists`] if a mod with `name` is already registered
/// * [`InstallationError::ModArchiveNotFound`] if `source` does not exist
/// * [`InstallationError::GamePath`] if the game path in the settings isn't the game folder
/// * [`InstallationError::UnsupportedCompression`] if the archive extension is unsupported
/// * [`InstallationError::ZipExtraction`], [`InstallationError::SevenZipExtraction`],
///   [`InstallationError::RarExtraction`] if the archive can't be extracted
/// * [`InstallationError::EntryReading`] if an entry in the mod folder cannot be read
/// * [`InstallationError::ModlessFolder`] if no valid mod files are recognized
/// * [`InstallationError::AmbiguousFiles`] if several files would be installed at the same place
/// * [`InstallationError::FileConflict`] if file conflicts are detected and overwriting is not allowed
/// * [`InstallationError::FilePlacing`] if a file can't be copied into the game folder
/// * [`InstallationError::Data`] if mod data cannot be saved to the data file
pub fn install_mod(
    source: &Path,
    name: &str,
    forced_overwrite: bool,
    settings: &Settings,
    data: &mut Mods,
    progress: &mut dyn FnMut(InstallStep, &str),
) -> Result<Mod, InstallationError> {
    progress(InstallStep::Checking, "Checking the mod, its name and the game folder");
    let name = name.trim();
    check_prerequisites(source, name, settings, data)?;

    let extracting = if source.is_dir() { "Reading the mod folder" } else { "Extracting the archive" };
    progress(InstallStep::Extracting, extracting);
    let mod_source = archive::prepare_source(source, &settings.extraction_dir(), settings.keep_extracted_folders)?;

    progress(InstallStep::Analysing, "Recognizing the mod files");
    let analysis = analysis::analyse(mod_source.path(), name)?;
    for warning in &analysis.warnings {
        progress(InstallStep::Warning, warning);
    }
    if analysis.mod_type == ModType::ReshadePreset && !is_reshade_installed(&settings.game_path) {
        progress(
            InstallStep::Warning,
            "ReShade doesn't seem to be installed in the game folder: the preset won't have any effect until it is (tested version: 4.7.0)",
        );
    }

    progress(InstallStep::CheckingConflicts, "Checking for conflicts with installed mods");
    let plan = plan::plan_installation(&analysis.files, &settings.game_path, data)?;
    if !plan.conflicts.is_empty() && must_refuse_conflicts(settings.files_conflict_resolution, forced_overwrite) {
        return Err(InstallationError::FileConflict(plan.conflicts));
    }

    progress(InstallStep::Installing, &format!("Installing {} file(s)", plan.files.len()));
    let mut new_data = data.clone();
    let mut tx = Transaction::new(&settings.game_path);
    let backups = place_files(&plan, &mut new_data, &mut tx, &settings.game_path, mod_source.is_disposable())?;

    let files = plan.files.into_iter().map(|f| f.destination).collect();
    let installed = Mod::new(name.to_string(), files, analysis.mod_type, backups);
    new_data.mods.push(installed.clone());

    progress(InstallStep::SavingData, "Saving the installed mods list");
    new_data.save()?;
    tx.commit();
    *data = new_data;

    Ok(installed)
}

/// Validates everything that can be checked before extracting the mod
fn check_prerequisites(source: &Path, name: &str, settings: &Settings, data: &Mods) -> Result<(), InstallationError> {
    if !is_valid_folder_name(name) {
        return Err(InstallationError::InvalidName(name.to_string()));
    }
    if data.name_exists(name) {
        return Err(InstallationError::NameAlreadyExists(name.to_string()));
    }
    if !source.exists() {
        return Err(InstallationError::ModArchiveNotFound(source.to_path_buf()));
    }
    validate_game_path(&settings.game_path)?;

    Ok(())
}

/// Evaluates conflict resolution settings and the overwrite flag
///
/// # Returns
/// * `true` if conflicts must block the installation
/// * `false` if conflicting files should be overwritten
fn must_refuse_conflicts(resolution: ConflictResolution, forced_overwrite: bool) -> bool {
    resolution == ConflictResolution::Warn && !forced_overwrite
}

/// Puts every planned file in the game folder
///
/// # Returns
/// * [`Ok`] -> The original game files that were set aside (and those inherited from the mods taken over)
fn place_files(
    plan: &InstallPlan,
    data: &mut Mods,
    tx: &mut Transaction,
    game_path: &Path,
    consume_sources: bool,
) -> Result<Vec<Backup>, InstallationError> {
    let mut backups = Vec::new();

    for file in &plan.files {
        match file.placement {
            Placement::Shared => continue,
            Placement::TakeOver(owner) => backups.extend(take_over(&mut data.mods[owner], &file.destination, tx)?),
            Placement::Free => {}
        }

        // Whatever is still there isn't tracked by any mod: it's an original game file
        if file.destination.exists() {
            let backup = unique_path(&backup_location(&file.destination, game_path));
            tx.move_file(&file.destination, &backup)
                .map_err(|source| InstallationError::FilePlacing {
                    from: file.destination.clone(),
                    to: backup.clone(),
                    source,
                })?;
            backups.push(Backup {
                original: file.destination.clone(),
                backup,
            });
        }

        tx.place_file(&file.source, &file.destination, consume_sources)
            .map_err(|source| InstallationError::FilePlacing {
                from: file.source.clone(),
                to: file.destination.clone(),
                source,
            })?;
    }

    Ok(backups)
}

/// Removes the file at `destination` from `owner`, deleting its copy of the file
///
/// # Returns
/// * [`Some`] -> The backup of the original game file `owner` had replaced, which the new mod inherits
/// * [`None`] -> `owner` hadn't replaced any original file (or had put it back because it was disabled)
fn take_over(owner: &mut Mod, destination: &Path, tx: &mut Transaction) -> Result<Option<Backup>, InstallationError> {
    let Some(index) = owner.files.iter().position(|f| same_path(&Mod::active_path(f), destination)) else {
        return Ok(None);
    };

    let owned_file = owner.files.remove(index);
    tx.discard_file(&owned_file)?;

    let backup = owner.take_backup_for(destination);
    // A disabled mod already put the original back at `destination`: it will be set aside again
    let was_enabled = same_path(&owned_file, destination);
    Ok(backup.filter(|_| was_enabled))
}

/// Errors that could occur during mod installation
#[derive(Error, Debug)]
pub enum InstallationError {
    /// The specified mod archive/folder was not found
    #[error("'{}' doesn't exist", .0.display())]
    ModArchiveNotFound(PathBuf),

    /// A mod with the given name is already tracked
    #[error("A mod with name '{0}' already exists")]
    NameAlreadyExists(String),

    /// The name can't be used (it is also the name of a folder for texture mods)
    #[error("'{0}' is not a valid mod name: it must not be empty, nor contain / \\ : * ? \" < > |")]
    InvalidName(String),

    /// The game folder in the settings is wrong
    #[error("{0}")]
    GamePath(#[from] GamePathError),

    /// The archive extension is not supported (supported types: `.zip`, `.7z`, `.rar`)
    #[error("'{}' is of an unsupported compression type (supported types are .zip, .7z .rar)", .0.display())]
    UnsupportedCompression(PathBuf),

    /// An I/O error occurred during file or folder access
    #[error("Couldn't access/open a file. {0}")]
    FileManaging(#[from] std::io::Error),

    /// An error occurred during ZIP archive extraction
    #[error("Couldn't extract zip file. {0}")]
    ZipExtraction(#[from] zip::result::ZipError),

    /// An error occurred during 7z archive extraction
    #[error("Couldn't extract 7z file. {0}")]
    SevenZipExtraction(#[from] sevenz_rust::Error),

    /// An error occurred during RAR archive extraction
    #[error("Couldn't extract rar file. {0}")]
    RarExtraction(#[from] unrar::error::UnrarError),

    /// An error occurred while traversing directory entries
    #[error("Couldn't read entry from mod folder. {0}")]
    EntryReading(#[from] walkdir::Error),

    /// The folder does not contain any recognized mod files
    #[error("'{}' doesn't contain any file ATA knows how to install", .0.display())]
    ModlessFolder(PathBuf),

    /// Several files of the mod would be installed at the same place
    #[error("{}", AmbiguousFilesDisplay(.0))]
    AmbiguousFiles(Vec<PathBuf>),

    /// Installation was blocked due to file conflicts with already installed mods
    #[error("{}", ConflictsDisplay(.0))]
    FileConflict(Vec<Conflict>),

    /// A file couldn't be put in place
    #[error("Couldn't move/copy '{}' to '{}'. {source}", from.display(), to.display())]
    FilePlacing {
        from: PathBuf,
        to: PathBuf,
        source: std::io::Error,
    },

    /// An error occurred during data file interaction
    #[error("{0}")]
    Data(#[from] DataInteractionError),

    /// A path component (extension/filename/stem/parent) could not be extracted from a path
    #[error("An error occurred while interacting with files. {0}")]
    FilesInteraction(#[from] FilesInteractionError),
}

/// Formats [`InstallationError::FileConflict`]
struct ConflictsDisplay<'a>(&'a [Conflict]);

impl fmt::Display for ConflictsDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Found file conflicts with already installed mods (install again with overwrite to replace them):")?;
        for conflict in self.0 {
            write!(f, "\n  '{}' (installed by '{}')", conflict.file.display(), conflict.owner)?;
        }
        Ok(())
    }
}

/// Formats [`InstallationError::AmbiguousFiles`]
struct AmbiguousFilesDisplay<'a>(&'a [PathBuf]);

impl fmt::Display for AmbiguousFilesDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Some files of the mod would be installed at the same place, the archive probably contains \
             alternative versions of the mod. Extract it and install the folder of the version you want:"
        )?;
        for file in self.0 {
            write!(f, "\n  '{}'", file.display())?;
        }
        Ok(())
    }
}
