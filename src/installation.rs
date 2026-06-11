//! **installation** is a module that contains the functions needed to install a mod
//!
//! This includes:
//! * **decompressing**: Going from the compressed archive to a normal folder
//! * **understanding the mod**: Looks at the type of files in the folder to understand what type of mod it is
//! * **installing the mod**: Moves the files found to be of the mod in the folder in which that mod type goes
//! * **updates the saved data**: Adds the newly installed mod to the data file (*~/.config/ATA/data.json*)
//!
//! Main function: [`install_mod`]



use crate::settings::{ConflictResolution, Settings};

use crate::data::{Data, DataInteractionError, Mod, ModType};

use std::fs::{File, copy, create_dir_all};

use std::path::{Path, PathBuf};

use std::collections::{HashMap, HashSet};

use thiserror::Error;

use walkdir::WalkDir;

use zip;

use sevenz_rust;

use unrar;

use chrono::Utc;



/// Installs a mod from a compressed archive and saves it in a config with user-decided name
///
/// # Arguments
/// * `compressed_mod_folder_path` - The path to the compressed archive which contains the mod
/// * `config` - The config to save the mod in
/// * `answered_name` - The name to give the mod
///
/// # Returns
/// * [`Ok`] -> The Mod installed
/// * [`Err`] -> The type of error that occured
///
/// # Errors
/// * [`InstallationError::FileNotFound`] if the path to the compressed archive leads to nothing
/// * [`InstallationError::ExtensionlessFile`] if the compressed archive has no extension
/// * [`InstallationError::InvalidExtension`] if the compressed archive has an extension that contains invalid UTF-8
/// * [`InstallationError::NamelessFile`] if the compressed archive has no name
/// * [`InstallationError::ParentlessFile`] if the path to the compressed archive is root ("/") or is an empty string
/// * [`InstallationError::UnsupportedCompression`] if the compressed archive has an extension that isn't supported (.zip, .7z, .rar)
/// * [`InstallationError::FileAccessing`] if a problem occurs during file/folder creation/deletion
/// * [`InstallationError::ZipExtracionFailed`] if a zip archive extraction fails
/// * [`InstallationError::SevenZipExtractionFailed`] if a 7z archive extraction fails
/// * [`InstallationError::RarExtractionFailed`] if a rar archive extraction fails
/// * [`InstallationError::EntryReading`] if one of the entries of the mod folder couldn't be read
/// * [`InstallationError::ModlessFolder`] if no mod was reckoned to be present in the folder
/// * [`InstallationError::InvalidFileName`] if one of the paths to a mod file is either root or ends in `..`
/// * [`InstallationError::FileCopying`] if a problem occurs during file copying
/// * [`InstallationError::DataSaving`] if the mod data couldn't be saved to the data file
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

    let conflicting_files = check_for_conflicts(mod_data.values().collect::<Vec<_>>(), data)?;
    let conflicts_present = conflicting_files.len() > 0;

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
                ModType::try_from(HashSet::from(mod_data.into_keys().collect::<HashSet<_>>()))?,
                Utc::now(),
            );

            data.save_new_mod(&installed_mod)
                .map_err(|er| InstallationError::Data(er))?;

            Ok(installed_mod)
        }
    }
}



/* ------------- */
/*   UTILITIES   */
/* ------------- */

#[derive(Error, Debug)]
pub enum InstallationError {
    #[error("'{0}' doesn't exist")]
    ModArchiveNotFound(PathBuf),

    #[error("A mod with name '{0}' already exists")]
    NameAlreadyExists(String),

    #[error("'{0}' is an extensionless file")]
    ExtensionlessFile(PathBuf),

    #[error("'{0}' contains invalid UTF-8 characters in its extension")]
    InvalidExtension(PathBuf),

    #[error("'{0}' is a nameless file")]
    NamelessFile(PathBuf),

    #[error("'{0}' contains invalid UTF-8 in its name")]
    InvalidFileName(PathBuf),

    #[error("'{0}' is a parentless file, it is either the root folder or an empty string")]
    ParentlessFile(PathBuf),

    #[error("'{0}' is of an unsupported compression type (supported types are .zip, .7z .rar)")]
    UnsupportedCompression(PathBuf),

    #[error("Couldn't access/open a file. {0}")]
    FileManaging(#[from] std::io::Error),

    #[error("Couldn't extract zip file. {0}")]
    ZipExtracion(#[from] zip::result::ZipError),

    #[error("Couldn't extract 7z file. {0}")]
    SevenZipExtraction(#[from] sevenz_rust::Error),

    #[error("Couldn't extract rar file. {0}")]
    RarExtraction(#[from] unrar::error::UnrarError),

    #[error("Couldn't read entry from mod folder. {0}")]
    EntryReading(#[from] walkdir::Error),

    #[error("The given folder doesn't contain a mod")]
    ModlessFolder(PathBuf),

    #[error("Found file conflicts with already installed mods: {0:?}")]
    FileConflict(HashMap<PathBuf, String>),

    #[error("A config related error occured. {0}")]
    Data(#[from] DataInteractionError),
}



/// Decompresses a compressed archive into a folder with the same name (zip, 7z, rar)
///
/// # Arguments
/// * `compressed_mod_folder` - The path to the compressed archive
///
/// # Returns
/// * [`Ok`] -> The path to the decompressed folder
/// * [`Err`] -> The error that occured
///
/// # Errors
/// * [`InstallationError::ExtensionlessFile`] if the compressed archive has no extension
/// * [`InstallationError::InvalidExtension`] if the compressed archive has an extension that contains invaid UTF-8
/// * [`InstallationError::NamelessFile`] if the compressed archive has no name or an invalid one
/// * [`InstallationError::ParentlessFile`] if the compressed archive is root or has no parent directory
/// * [`InstallationError::UnsupportedCompression`] if the compressed archive has an extension that isn't supported (.zip, .7z, .rar)
/// * [`InstallationError::FileAccessing`] if problem occur during file/folder creation/deletion
/// * [`InstallationError::ZipExtracionFailed`] if the zip archive extraction fails
/// * [`InstallationError::SevenZipExtractionFailed`] if the 7z archive extraction fails
/// * [`InstallationError::RarExtractionFailed`] if the rar archive extraction fails
fn decompress_folder(compressed_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
    let extension = get_file_extension(compressed_mod_folder)?;

    let folder_name = compressed_mod_folder
        .file_stem()
        .ok_or(InstallationError::NamelessFile(compressed_mod_folder.to_path_buf()))?
        .to_str()
        .ok_or(InstallationError::InvalidFileName(compressed_mod_folder.to_path_buf()))?;
    let mod_folder_parent =
        compressed_mod_folder
            .parent()
            .ok_or(InstallationError::ParentlessFile(
                compressed_mod_folder.to_path_buf(),
            ))?;
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

/// Decompresses a zip archive
///
/// # Arguments
/// * `zipped_mod_folder` - The path to the zip archive
/// * `zip_extraction_folder` - The path to the folder to extract the zip archive into
///
/// # Returns
/// * [`Ok`] -> The path to the decompressed folder
/// * [`Err`] -> The error that occured
///
/// # Errors
/// * [`InstallationError::FileAccessing`] if problem occur during file/folder creation/deletion
/// * [`InstallationError::ZipExtracionFailed`] if the zip archive extraction fails
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
/// * `sevzipped_mod_folder` - The path to the 7z archive
/// * `sevzip_extraction_folder` - The path to the folder to extract the 7z archive into
///
/// # Returns
/// * [`Ok`] -> The path to the decompressed folder
/// * [`Err`] -> The error that occured
///
/// # Errors
/// * [`InstallationError::SevenZipExtractionFailed`] if the 7z archive extraction fails
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
/// * `rared_mod_folder` - The path to the RAR archive
/// * `rar_extraction_folder` - The path to the folder to extract the RAR archive into
///
/// # Returns
/// * [`Ok`] -> The path to the decompressed folder
/// * [`Err`] -> The error that occured
///
/// # Errors
/// * [`InstallationError::RarExtractionFailed`] if the RAR archive extraction fails
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



/// Returns the mod type and the filtered files of the mod
///
/// # Arguments
/// * `mod_folder_path` - The path to the mod folder
///
/// # Returns
/// * [`Ok`] -> [`Option`] -< [`Some`] -> the mod type and filtered files of the mod / [`None`] -> no mod found
/// * [`Err`] -> The error that occured
///
/// # Errors
/// * [`InstallationError::EntryReading`] if one of the entries of the mod folder couldn't be read
/// * [`InstallationError::FileAccessing`] if problem occur during file/folder creation/deletion
fn get_mod_data(
    mod_folder_path: &Path,
) -> Result<Option<HashMap<ModType, PathBuf>>, InstallationError> {
    let mut mod_files: HashMap<ModType, PathBuf> = HashMap::new();

    for entry in WalkDir::new(&mod_folder_path) {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        if current_entry.file_type().is_dir() {
            continue;
        }

        let extension = match get_file_extension(entry_path) {
            Ok(ext) => ext,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
        if !ModType::all_extensions().contains(extension) {
            continue;
        }

        let prefix: String = get_file_name(entry_path)?.chars().take(2).collect();

        if let Some(entry_mod_type) = ModType::try_from((extension, prefix.as_ref())).ok() {
            mod_files.insert(entry_mod_type, entry_path.to_path_buf());
        }
    }

    match mod_files.len() > 0 {
        true => Ok(Some(mod_files)),
        false => Ok(None)
    }
}



fn check_for_conflicts<'a>(
    mod_files: Vec<&PathBuf>,
    data: &'a Data,
) -> Result<HashMap<PathBuf, String>, InstallationError> {
    let mut installed: HashMap<&str, &'a String> = HashMap::new();

    for m in &data.mods {
        for f in &m.files {
            installed.insert(get_file_name(f)?, &m.name);
        }
    }

    let mut conflicts: HashMap<PathBuf, String> = HashMap::new();
    for f in mod_files {
        let name = get_file_name(&f)?;
        if let Some(mod_name) = installed.get(name) {
            conflicts.insert(f.clone(), mod_name.to_string());
        }
    }

    Ok(conflicts)
}

fn get_warning_necessity(warn_setting: ConflictResolution, overwrite_flag: bool) -> bool {
    let warn: bool = match (warn_setting, overwrite_flag) {
        (ConflictResolution::Warn, false) => true,
        (ConflictResolution::Warn, true) => false,
        (ConflictResolution::Overwrite, _) => false,
    };

    warn
}



/// Installs the mod files to the game folder
///
/// # Arguments
/// * `mod_type` - The type of mod to install
/// * `mod_files` - The paths of the files of the mod to install
/// * `game_path` - The path to the game folder
///
/// # Returns
/// * [`Ok`] -> The paths of the installed files
/// * [`Err`] -> The error that occured
///
/// # Errors
/// * [`InstallationError::FileAccessing`] if a problem occurs during file/folder creation/deletion
/// * [`InstallationError::InvalidFileName`] if one of the paths to a mod file is either root or ends in `..`
/// * [`InstallationError::FileCopying`] if a problem occurs during file copying
fn install(
    mod_files: &HashMap<ModType, PathBuf>,
    mod_name: &String,
    game_path: &PathBuf,
) -> Result<Vec<PathBuf>, InstallationError> {
    let mut copied_files = vec![];
    
    for file in mod_files {
        let installation_folder = game_path.join(file.0.get_corresponding_folder(mod_name));

        copied_files.push(copy_mod_fle(file.1, installation_folder)?);
    }

    Ok(copied_files)
}

/// Copies the mod files to the destination folder
///
/// # Arguments
/// * `mod_files` - The paths of the mod files to copy
/// * `destination_folder_path` - The path to the destination folder
///
/// # Returns
/// * [`Ok`] -> The paths of the copied files
/// * [`Err`] -> The error that occured
///
/// # Errors
/// * [`InstallationError::FileAccessing`] if a problem occurs during file/folder creation/deletion
/// * [`InstallationError::InvalidFileName`] if one of the paths to a mod file is either root or ends in `..`
/// * [`InstallationError::FileCopying`] if a problem occurs during file copying
fn copy_mod_fle(
    mod_file: &PathBuf,
    destination_folder: PathBuf,
) -> Result<PathBuf, InstallationError> {
    create_dir_all(&destination_folder)?;

    let filename = get_file_name(&mod_file)?;

    let copied_file = destination_folder.join(filename);
    copy(mod_file, &copied_file)?;

    Ok(copied_file)
}



/// Returns the file extension of a path as a string slice
///
/// # Arguments
/// * `path` - The path to get the extension from
///
/// # Returns
/// * [`Ok`] -> The file extension as a string slice
/// * [`Err`] -> The error that occured
///
/// # Errors
/// * [`InstallationError::ExtensionlessFile`] if the compressed archive has no extension
/// * [`InstallationError::InvalidExtension`] if the compressed archive has an invalid extension
fn get_file_extension(path: &Path) -> Result<&str, InstallationError> {
    let Some(extension) = path.extension() else {
        return Err(InstallationError::ExtensionlessFile(path.to_path_buf()));
    };
    let Some(extension_str) = extension.to_str() else {
        return Err(InstallationError::InvalidExtension(path.to_path_buf()));
    };

    Ok(extension_str)
}

fn get_file_name(path: &Path) -> Result<&str, InstallationError> {
    let name = path
        .file_name()
        .ok_or_else(|| InstallationError::NamelessFile(path.to_path_buf()))?;

    name.to_str()
        .ok_or_else(|| InstallationError::InvalidFileName(path.to_path_buf()))
}
