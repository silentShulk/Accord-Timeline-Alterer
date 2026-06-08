//! **installation** is a module that contains the functions needed to install a mod
//! 
//! This includes:
//! * **decompressing**: Going from the compressed archive to a normal folder
//! * **understanding the mod**: Looks at the type of files in the folder to understand what type of mod it is
//! * **installing the mod**: Moves the files found to be of the mod in the folder in which that mod type goes
//! * **updates the saved data**: Adds the newly installed mod to the data file (*~/.config/ATA/data.json*) 
//! 
//! Main function: [`install_mod`]



use std::collections::HashMap;
use std::fs::{File, copy, create_dir_all, remove_file};

use std::path::{PathBuf, Path};

use thiserror::Error;

use walkdir::WalkDir;

use zip::ZipArchive;

use sevenz_rust::decompress_file;

use unrar::Archive;

use crate::data::{Data, DataInteractionError, Mod, ModType};

use chrono::Utc;

use crate::settings::{ConflictResolution, Settings};



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
pub fn install_mod(compressed_mod_folder_path: &Path, answered_name: String, forced_overwrite: bool, settings: &Settings, data: &mut Data) -> Result<Option<Mod>, InstallationError> {
    if !compressed_mod_folder_path.exists() {
        return Err(InstallationError::FileNotFound(compressed_mod_folder_path.to_path_buf()));
    }
    if data.name_exists(&answered_name) {
        return Err(InstallationError::Config(DataInteractionError::ModNameExists(answered_name)))
    }

    let mut mod_folder_path = decompress_folder(&compressed_mod_folder_path)?;

    let mod_data = get_mod_data(&mut mod_folder_path)?
       	.ok_or(InstallationError::ModlessFolder(mod_folder_path.clone()))?;

    let conflicting_files = check_for_conflicts(&mod_data.1, data);
    let need_to_warn = match (settings.files_conflict_resolution, forced_overwrite) {
        (ConflictResolution::Warn, false) => true,
        (ConflictResolution::Warn, true) => false,
        (ConflictResolution::Overwrite, _) => false
    };
    let early_stop =
        conflicting_files.len() > 0
        &&
        need_to_warn;

    if early_stop {
        Ok(None)
    } else {
        if conflicting_files.len() > 0 {
            data.remove_conflicts(conflicting_files);
        }
        let mod_files = install(&mod_data.0, &answered_name, &mod_data.1, &settings.game_path)?;
        let installed_mod = Mod::new(answered_name.clone(), mod_files, true, mod_data.0, Utc::now());
    
       	data.save_new_mod(&installed_mod)
            .map_err(|er| InstallationError::Config(er))?;
    
        Ok(Some(installed_mod))
    }
}



/* ------------- */
/*   UTILITIES   */
/* ------------- */

#[derive(Error, Debug)]
pub enum InstallationError {
    // Check if path exists
    #[error("{0} doesn't exist")]
    FileNotFound(PathBuf),
    
    // Decompresss archive
    #[error("{0} is an extensionless file")]
    ExtensionlessFile(PathBuf),
    
    #[error("{0} contains invalid UTF-8 characters in its extension")]
    InvalidExtension(PathBuf),
    
    #[error("{0} is a nameless file")]
    NamelessFile(PathBuf),
    
    #[error("{0} is a parentless file (root folder) or is an empty string")]
    ParentlessFile(PathBuf),
    
    #[error("{0} is of an unsupported compression type (supported types are .zip, .7z .rar)")]
    UnsupportedCompression(PathBuf),
    
    #[error("Couldn't access/open a file. {0}")]
    FileAccessing(#[from] std::io::Error),
    
    #[error("Couldn't extract zip file {0}")]
    ZipExtracionFailed(#[from] zip::result::ZipError),
    
    #[error("Couldn't extract 7z file {0}")]
    SevenZipExtractionFailed(#[from] sevenz_rust::Error),
    
    #[error("Couldn't extract rar file {0}")]
    RarExtractionFailed(#[from] unrar::error::UnrarError),
    
    // Get mod data
    #[error("Couldn't read entry from mod folder. {0}")]
    EntryReading(#[from] walkdir::Error),
    
    #[error("The given folder doesn't contain a mod")]
    ModlessFolder(PathBuf),
    
    // File copying
    #[error("{0} is either root or ends in ..")]
    InvalidFileName(PathBuf),
    
    #[error("Couldn't copy {0} to {1}. {2}")]
    FileCopying(PathBuf, PathBuf, std::io::Error),
    
    // Data saving
    #[error("A config related error occured. {0}")]
    Config(#[from] DataInteractionError),
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

   	let folder_name = compressed_mod_folder.file_stem()
           .ok_or(InstallationError::NamelessFile(compressed_mod_folder.to_path_buf()))?;
	let mod_folder_parent = compressed_mod_folder.parent()
		.ok_or(InstallationError::ParentlessFile(compressed_mod_folder.to_path_buf()))?;
    let target_folder = mod_folder_parent
           .join(folder_name);

    match extension {
        "zip" => decompress_zip(compressed_mod_folder, target_folder),
        "7z" => decompress_7z(compressed_mod_folder, target_folder),
        "rar" => decompress_rar(compressed_mod_folder, target_folder),
        _ => Err(InstallationError::UnsupportedCompression(compressed_mod_folder.to_path_buf())),
    }
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
fn decompress_zip(zipped_mod_folder: &Path, zip_extraction_folder: PathBuf) -> Result<PathBuf, InstallationError> {
    let zip_file = File::open(zipped_mod_folder)?;
    let mut zip_archive = ZipArchive::new(zip_file)?;

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
fn decompress_7z(sevzipped_mod_folder: &Path, sevzip_extraction_folder: PathBuf) -> Result<PathBuf, InstallationError> {
	decompress_file(sevzipped_mod_folder, &sevzip_extraction_folder)?;

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
fn decompress_rar(rared_mod_folder: &Path, rar_extraction_folder: PathBuf) -> Result<PathBuf, InstallationError> {
	let mut rar_archive = Archive::new(rared_mod_folder).open_for_processing()?;

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
fn get_mod_data(mod_folder_path: &Path) -> Result<Option<(ModType, Vec<PathBuf>)>, InstallationError> {
    let mut mod_contained: Option<ModType> = None;
    let mut mod_files: Option<Vec<PathBuf>> = None;

    for entry in WalkDir::new(&mod_folder_path) {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        if !current_entry.file_type().is_file() {
            continue;
        }

        let extension = match get_file_extension(entry_path) {
            Ok(ext) => ext,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };

        if extension != "dds" && extension != "dtt" && extension != "dat" && extension != "usm" {
        	remove_file(entry_path)?;
         	continue;
        }

        mod_files.get_or_insert_with(Vec::new).push(entry_path.to_path_buf());

        if mod_contained.is_none() {
            mod_contained = match extension {
                "dll" => Some(ModType::DLL),
                "dds" => Some(ModType::Textures),
                "dtt" | "dat" => {
                    let name = entry_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name.starts_with("pl") { Some(ModType::PlayerModels) }
                    else if name.starts_with("wp") { Some(ModType::WeaponModels) }
                    else if name.starts_with("bg") { Some(ModType::WorldModels) }
                    else { None }
                }
                "usm" => Some(ModType::CutsceneReplacements),
                "ini" => Some(ModType::ReshadePreset),
                _ => None,
            };
        }
    }

    Ok(mod_contained.zip(mod_files))
}



fn check_for_conflicts(mod_files: &[PathBuf], data: &Data) -> Vec<(String, PathBuf)> {
    let installed: HashMap<&std::ffi::OsStr, (&str, &PathBuf)> = data.mods.iter()
        .flat_map(|m| m.files.iter().filter_map(move |f| {
            f.file_name().map(|name| (name, (m.name.as_str(), f)))
        }))
        .collect();

    mod_files.iter()
        .filter_map(|file| {
            file.file_name()
                .and_then(|name| installed.get(name))
                .map(|(mod_name, installed_path)| (mod_name.to_string(), (*installed_path).clone()))
        })
        .collect()
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
fn install(mod_type: &ModType, mod_name: &String, mod_files: &Vec<PathBuf>, game_path: &PathBuf) -> Result<Vec<PathBuf>, InstallationError> {
    let mut installation_folder = game_path.join(mod_type.get_corresponding_folder());

    if mod_type == &ModType::Textures {
        installation_folder = installation_folder.join(mod_name);
    }

    
    copy_mod_files(mod_files, PathBuf::from(installation_folder))
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
fn copy_mod_files(mod_files: &Vec<PathBuf>, destination_folder_path: PathBuf) -> Result<Vec<PathBuf>, InstallationError> {
    create_dir_all(&destination_folder_path)?;

    let mut copied_files: Vec<PathBuf> = vec![];
    for file in mod_files {
       	let Some(filename) = file.file_name() else {
            return Err(InstallationError::InvalidFileName(file.to_path_buf()))
        };
        
        let copied_file = destination_folder_path.join(filename); 
        copy(file, &copied_file)
            .map_err(|er| InstallationError::FileCopying(file.as_path().to_path_buf(), destination_folder_path.clone(), er))?;
        
        copied_files.push(copied_file);
    }

    Ok(copied_files)
}
