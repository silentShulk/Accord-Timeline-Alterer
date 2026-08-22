//! **installation** is a module that contains the functions needed to install a mod
//!
//! This includes:
//! * **decompressing**: Going from the compressed archive to a normal folder
//! * **understanding the mod**: Looks at the type of files in the folder to understand what type of mod it is
//! * **installing the mod**: Moves the files found to be of the mod in the folder in which that mod type goes
//! * **updates the saved data**: Adds the newly installed mod to the data file (*~/.config/ATA/data.json*)
//!
//! Main function: [`install_mod`]

use crate::utils::files::{get_extension_or_err, get_filename_or_err, get_filestem_or_err, get_parent_or_err, FilesInteractionError};
use crate::data::{Data, DataInteractionError, Mod, ModType};
use crate::settings::{ConflictResolution, Settings};

use std::fs::{File, copy, create_dir_all};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

use thiserror::Error;
use walkdir::WalkDir;
use zip;
use sevenz_rust;
use unrar;
use chrono::Utc;



/// Installs a mod from a compressed archive and saves it in data with user-decided name
///
/// # Arguments
/// * `compressed_mod_folder_path` - Path to the compressed archive containing the mod
/// * `answered_name` - Name to assign to the mod
/// * `forced_overwrite` - Whether to force overwriting existing conflicting mod files
/// * `settings` - Runtime application settings
/// * `data` - Mutable reference to the current saved [`Data`] state
///
/// # Returns
/// * [`Ok`] -> The installed [`Mod`]
/// * [`Err`] -> The type of error that occurred
///
/// # Errors
/// * [`InstallationError::ModArchiveNotFound`] if the path to the archive does not exist
/// * [`InstallationError::NameAlreadyExists`] if a mod with `answered_name` is already registered
/// * [`InstallationError::FilesInteraction`] if a path component (extension/filename/stem/parent) could not be extracted
/// * [`InstallationError::UnsupportedCompression`] if the archive extension is unsupported
/// * [`InstallationError::FileManaging`] if a file or directory operation fails
/// * [`InstallationError::ZipExtracion`] if ZIP archive extraction fails
/// * [`InstallationError::SevenZipExtraction`] if 7z archive extraction fails
/// * [`InstallationError::RarExtraction`] if RAR archive extraction fails
/// * [`InstallationError::EntryReading`] if an entry in the mod folder cannot be read
/// * [`InstallationError::ModlessFolder`] if no valid mod files are recognized
/// * [`InstallationError::FileConflict`] if file conflicts are detected and overwriting is disabled
/// * [`InstallationError::Data`] if mod data cannot be saved to the data file
pub fn install_mod(
    compressed_mod_folder_path: &Path,
    answered_name: String,
    forced_overwrite: bool,
    settings: &Settings,
    data: &mut Data,
) -> Result<Mod, InstallationError> {
    if !compressed_mod_folder_path.exists() {
        return Err(InstallationError::ModArchiveNotFound(
            compressed_mod_folder_path.to_path_buf(),
        ));
    }
    if data.name_exists(&answered_name) {
        return Err(InstallationError::NameAlreadyExists(answered_name));
    }

    let mut mod_folder_path = decompress_folder(&compressed_mod_folder_path)?;

    let mod_data = get_mod_data(&mut mod_folder_path)?
        .ok_or(InstallationError::ModlessFolder(mod_folder_path.clone()))?;

    let conflicting_files = check_for_conflicts(mod_data.keys().collect::<Vec<_>>(), data)?;
    let conflicts_present = !conflicting_files.is_empty();

    let should_install: bool = match (
        conflicts_present,
        get_warning_necessity(settings.files_conflict_resolution, forced_overwrite),
    ) {
        (true, true) => false,
        (true, false) => {
            data.remove_conflicts(&conflicting_files);
            true
        }
        (false, _) => true,
    };

    match should_install {
        false => Err(InstallationError::FileConflict(conflicting_files)),
        true => {
            let installed_files = install(
                &mod_data,
                &answered_name,
                &settings.game_path,
            )?;
            let installed_mod = Mod::new(
                answered_name.clone(),
                installed_files,
                true,
                ModType::try_from(mod_data.into_values().collect::<HashSet<_>>())?,
                Utc::now(),
            );

            data.save_new_mod(&installed_mod)?;

            if !settings.keep_extracted_folders {
                if let Err(cleanup_err) = std::fs::remove_dir_all(&mod_folder_path) {
                    eprintln!(
                        "Couldn't remove extracted temp folder '{}'. {}",
                        mod_folder_path.display(),
                        cleanup_err
                    );
                }
            }

            Ok(installed_mod)
        }
    }
}

/* ------------- */
/*   UTILITIES   */
/* ------------- */

/// Errors that could occur during mod installation
#[derive(Error, Debug)]
pub enum InstallationError {
    /// The specified compressed mod archive path was not found
    #[error("'{0}' doesn't exist")]
    ModArchiveNotFound(PathBuf),

    /// A mod with the given name is already tracked
    #[error("A mod with name '{0}' already exists")]
    NameAlreadyExists(String),

    /// The archive extension is not supported (supported types: `.zip`, `.7z`, `.rar`)
    #[error("'{0}' is of an unsupported compression type (supported types are .zip, .7z .rar)")]
    UnsupportedCompression(PathBuf),

    /// An I/O error occurred during file or folder access
    #[error("Couldn't access/open a file. {0}")]
    FileManaging(#[from] std::io::Error),

    /// An error occurred during ZIP archive extraction
    #[error("Couldn't extract zip file. {0}")]
    ZipExtracion(#[from] zip::result::ZipError),

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
    #[error("The given folder doesn't contain a mod")]
    ModlessFolder(PathBuf),

    /// Installation was blocked due to file conflicts with already installed mods
    #[error("Found file conflicts with already installed mods: {0:?}")]
    FileConflict(HashMap<PathBuf, String>),

    /// An error occurred during data file interaction
    #[error("A config related error occured. {0}")]
    Data(#[from] DataInteractionError),

    /// A path component (extension/filename/stem/parent) could not be extracted from a path
    #[error("An error occurred while interacting with files. {0}")]
    FilesInteraction(#[from] FilesInteractionError),
}

/// Decompresses a compressed archive into a folder with the same name (zip, 7z, rar)
///
/// # Arguments
/// * `compressed_mod_folder` - Path to the compressed archive
///
/// # Returns
/// * [`Ok`] -> Path to the decompressed folder
/// * [`Err`] -> The error that occurred
///
/// # Errors
/// * [`InstallationError::FilesInteraction`] if the archive path has no extension, filestem, or parent
/// * [`InstallationError::UnsupportedCompression`] if the archive type is unsupported
/// * [`InstallationError::FileManaging`] if directory creation or deletion fails
/// * [`InstallationError::ZipExtracion`] if ZIP extraction fails
/// * [`InstallationError::SevenZipExtraction`] if 7z extraction fails
/// * [`InstallationError::RarExtraction`] if RAR extraction fails
fn decompress_folder(compressed_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
    let extension = get_extension_or_err(compressed_mod_folder)?;

    let folder_name = get_filestem_or_err(compressed_mod_folder)?;
    let mod_folder_parent = get_parent_or_err(&compressed_mod_folder)?;
    let target_folder = mod_folder_parent.join(folder_name);

    if target_folder.exists() {
        std::fs::remove_dir_all(&target_folder)?;
    }

    match extension {
        "zip" => decompress_zip(compressed_mod_folder, target_folder),
        "7z" => decompress_7z(compressed_mod_folder, target_folder),
        "rar" => decompress_rar(compressed_mod_folder, target_folder),
        _ => Err(InstallationError::UnsupportedCompression(
            compressed_mod_folder.to_path_buf(),
        )),
    }
}

/// Decompresses a ZIP archive
///
/// # Arguments
/// * `zipped_mod_folder` - Path to the ZIP archive
/// * `zip_extraction_folder` - Destination folder for extraction
///
/// # Returns
/// * [`Ok`] -> Path to the extracted folder
/// * [`Err`] -> The error that occurred
///
/// # Errors
/// * [`InstallationError::FileManaging`] if opening the archive fails
/// * [`InstallationError::ZipExtracion`] if extraction fails
fn decompress_zip(
    zipped_mod_folder: &Path,
    zip_extraction_folder: PathBuf,
) -> Result<PathBuf, InstallationError> {
    let zip_file = File::open(zipped_mod_folder)?;
    let mut zip_archive = zip::ZipArchive::new(zip_file)?;

    zip_archive.extract(&zip_extraction_folder)?;

    Ok(zip_extraction_folder)
}

/// Decompresses a 7z archive
///
/// # Arguments
/// * `sevzipped_mod_folder` - Path to the 7z archive
/// * `sevzip_extraction_folder` - Destination folder for extraction
///
/// # Returns
/// * [`Ok`] -> Path to the extracted folder
/// * [`Err`] -> The error that occurred
///
/// # Errors
/// * [`InstallationError::SevenZipExtraction`] if 7z extraction fails
fn decompress_7z(
    sevzipped_mod_folder: &Path,
    sevzip_extraction_folder: PathBuf,
) -> Result<PathBuf, InstallationError> {
    sevenz_rust::decompress_file(sevzipped_mod_folder, &sevzip_extraction_folder)?;

    Ok(sevzip_extraction_folder)
}

/// Decompresses a RAR archive
///
/// # Arguments
/// * `rared_mod_folder` - Path to the RAR archive
/// * `rar_extraction_folder` - Destination folder for extraction
///
/// # Returns
/// * [`Ok`] -> Path to the extracted folder
/// * [`Err`] -> The error that occurred
///
/// # Errors
/// * [`InstallationError::RarExtraction`] if RAR processing or extraction fails
fn decompress_rar(
    rared_mod_folder: &Path,
    rar_extraction_folder: PathBuf,
) -> Result<PathBuf, InstallationError> {
    let mut rar_archive = unrar::Archive::new(rared_mod_folder).open_for_processing()?;

    while let Some(header) = rar_archive.read_header()? {
        rar_archive = if header.entry().is_file() {
            header.extract_to(&rar_extraction_folder)?
        } else {
            header.skip()?
        };
    }

    Ok(rar_extraction_folder)
}

/// Inspects extracted files to infer their mod type and filter relevant asset files
///
/// # Arguments
/// * `mod_folder_path` - Path to the decompressed mod folder
///
/// # Returns
/// * [`Ok`] -> [`Option`] containing a map of file paths to their detected [`ModType`], or [`None`] if no mod files found
/// * [`Err`] -> The error that occurred
///
/// # Errors
/// * [`InstallationError::EntryReading`] if reading directory entries fails
/// * [`InstallationError::FilesInteraction`] if a recognized entry's filename cannot be extracted
pub fn get_mod_data(
    mod_folder_path: &Path,
) -> Result<Option<HashMap<PathBuf, ModType>>, InstallationError> {
    let mut mod_files: HashMap<PathBuf, ModType> = HashMap::new();

    for entry in WalkDir::new(&mod_folder_path) {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        if current_entry.file_type().is_dir() {
            continue;
        }

        let extension = match get_extension_or_err(entry_path) {
            Ok(ext) => ext,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
        if !ModType::all_extensions().contains(extension) {
            continue;
        }

        let prefix: String = get_filename_or_err(entry_path)?.chars().take(2).collect();

        if let Ok(entry_mod_type) = ModType::try_from((extension, prefix.as_ref())) {
            mod_files.insert(entry_path.to_path_buf(), entry_mod_type);
        }
    }

    if !mod_files.is_empty() {
        Ok(Some(mod_files))
    } else {
        Ok(None)
    }
}

/// Checks incoming mod files against existing installed mods for filename collisions
///
/// # Arguments
/// * `mod_files` - List of file paths belonging to the incoming mod
/// * `data` - Reference to current saved [`Data`] state
///
/// # Returns
/// * [`Ok`] -> Map of incoming file paths to the names of currently installed mods they conflict with
/// * [`Err`] -> The error that occurred
///
/// # Errors
/// * [`InstallationError::FilesInteraction`] if a filename (installed or incoming) cannot be extracted
fn check_for_conflicts<'a>(
    mod_files: Vec<&PathBuf>,
    data: &'a Data,
) -> Result<HashMap<PathBuf, String>, InstallationError> {
    let mut installed: HashMap<&str, &'a String> = HashMap::new();

    for m in &data.mods {
        for f in &m.files {
            installed.insert(get_filename_or_err(f)?, &m.name);
        }
    }

    let mut conflicts: HashMap<PathBuf, String> = HashMap::new();
    for f in mod_files {
        let name = get_filename_or_err(&f)?;
        if let Some(mod_name) = installed.get(name) {
            conflicts.insert(f.clone(), mod_name.to_string());
        }
    }

    Ok(conflicts)
}

/// Evaluates conflict resolution settings and CLI flags to determine if confirmation/warning is required
///
/// # Arguments
/// * `warn_setting` - User configuration for handling conflicts ([`ConflictResolution`])
/// * `overwrite_flag` - CLI flag explicitly requesting an overwrite
///
/// # Returns
/// * `true` if a conflict warning must be produced (blocking installation)
/// * `false` if files should be overwritten directly
fn get_warning_necessity(warn_setting: ConflictResolution, overwrite_flag: bool) -> bool {
    match (warn_setting, overwrite_flag) {
        (ConflictResolution::Warn, false) => true,
        (ConflictResolution::Warn, true) => false,
        (ConflictResolution::Overwrite, _) => false,
    }
}

/// Copies mod files into their designated game folders according to their mod type
///
/// # Arguments
/// * `mod_files` - Map of source mod file paths to their [`ModType`]
/// * `mod_name` - Name assigned to the mod
/// * `game_path` - Path to the NieR:Automata game root directory
///
/// # Returns
/// * [`Ok`] -> Vector of destination [`PathBuf`]s where files were copied
/// * [`Err`] -> The error that occurred
///
/// # Errors
/// * [`InstallationError::FileManaging`] if creating directories or copying files fails
/// * [`InstallationError::FilesInteraction`] if a source filename cannot be extracted
fn install(
    mod_files: &HashMap<PathBuf, ModType>,
    mod_name: &String,
    game_path: &PathBuf,
) -> Result<Vec<PathBuf>, InstallationError> {
    let mut copied_files = vec![];

    for file in mod_files {
        let prefix: String = get_filename_or_err(file.0)?.chars().take(2).collect();

        let installation_folder = game_path.join(file.1.get_corresponding_folder(mod_name, prefix.as_ref()));

        copied_files.push(copy_mod_file(file.0, installation_folder)?);
    }

    Ok(copied_files)
}

/// Copies a single mod file into a target directory
///
/// # Arguments
/// * `mod_file` - Source file path
/// * `destination_folder` - Destination directory path
///
/// # Returns
/// * [`Ok`] -> Path to the copied file in its destination directory
/// * [`Err`] -> The error that occurred
///
/// # Errors
/// * [`InstallationError::FileManaging`] if directory creation or file copying fails
/// * [`InstallationError::FilesInteraction`] if `mod_file`'s filename cannot be extracted
fn copy_mod_file(
    mod_file: &PathBuf,
    destination_folder: PathBuf,
) -> Result<PathBuf, InstallationError> {
    create_dir_all(&destination_folder)?;

    let filename = get_filename_or_err(&mod_file)?;

    let copied_file = destination_folder.join(filename);
    copy(mod_file, &copied_file)?;

    Ok(copied_file)
}