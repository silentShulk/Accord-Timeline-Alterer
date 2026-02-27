use std::fs::{copy, read_dir, File};

use std::path::{PathBuf, Path};

use thiserror::Error;

use walkdir::WalkDir;

use zip::ZipArchive;

use sevenz_rust::decompress_file;

use unrar::Archive;

use crate::data_saving::{Mod, ModType};

use crate::user_interactions::ask_mod_name;



/* ------------- */
/*   UTILITIES   */
/* ------------- */

#[derive(Error, Debug)]
pub enum InstallationError {
    // Extension Reading
    #[error("{0} is an an extensionless file, it will be skipped")]
    ExtensionlessFileError(PathBuf),

    #[error("{0} has an extension containing invalid UTF-8, it will be skipped")]
    InvalidExtensionError(PathBuf),
    
    // Decompression
    #[error("The received compressed folder ({0}) uses an unsupported extension")]
    UnsupportedCompressionError(String),
    
    #[error("Couldn't access a {0}")]
    FileAccessingError(PathBuf),
    
    #[error("Couldn't extract zip archive. {0}")]
    FailedZipExtractionError(#[from] zip::result::ZipError),
    
    #[error("Compressed archive ({0}) doesn't have a parent directory")]
    ParentlessArchiveError(PathBuf),
    
    #[error("The compressed archive ({0}) doesn't have a name")]
    NamelessArchiveError(PathBuf),
       
    #[error("Couldn't extract 7z archive. {0}")]
    Failed7zExtractionError(#[from] sevenz_rust::Error),
    
    #[error("Couldn't extract rar archive. {0}")]
    FailedRarExtractionError(#[from] unrar::error::UnrarError),

    // Directory Reading Errors
    #[error("Couldn't read an entry. {0}")]
    EntryReadingError(#[from] walkdir::Error),
    
    #[error("Couldn't find or read completely {0}")]
    DirectoryReadingError(PathBuf, std::io::Error),

    
    #[error("Couldn't copy {0} to {1}. {2}")]
    FileCopyingError(PathBuf, PathBuf, std::io::Error),

    // Console interaction
    #[error("Encountered an error while trying to read/write the console. {0}")]
    ConsoleInteractionError(#[from] std::io::Error),
}



/* ------------- */
/*   UTILITIES   */
/* ------------- */

pub fn decompress_folder(compressed_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
    let extension = get_file_extension(compressed_mod_folder)?;
    
    match extension {
        ".zip" => decompress_zip(compressed_mod_folder),
        ".7z" => decompress_7z(compressed_mod_folder),
        ".rar" => decompress_rar(compressed_mod_folder),
        _ => Err(InstallationError::UnsupportedCompressionError(extension.to_string()))
    }
}

fn get_file_extension(path: &Path) -> Result<&str, InstallationError> {
    let Some(extension) = path.extension() else {
        return Err(InstallationError::ExtensionlessFileError(path.to_path_buf()));
    };
    let Some(extension_str) = extension.to_str() else {
       	return Err(InstallationError::InvalidExtensionError(path.to_path_buf()));
    };

    Ok(extension_str)
}

fn decompress_zip(zipped_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
	let folder_name = zipped_mod_folder.file_stem()
        .ok_or(InstallationError::NamelessArchiveError(zipped_mod_folder.to_path_buf()))?;
	let mod_folder_parent = zipped_mod_folder.parent()
		.ok_or(InstallationError::ParentlessArchiveError(zipped_mod_folder.to_path_buf()))?;
    let extracted_folder = mod_folder_parent
        .join(folder_name);
	
    let zip_file = File::open(zipped_mod_folder)
        .map_err(|_| InstallationError::FileAccessingError(zipped_mod_folder.to_path_buf()))?;
    let mut zip_archive = ZipArchive::new(zip_file)?;
    
    zip_archive.extract(mod_folder_parent)?;
    
    Ok(extracted_folder)
}

fn decompress_7z(sevzipped_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
	let folder_name = sevzipped_mod_folder.file_stem()
        .ok_or(InstallationError::NamelessArchiveError(sevzipped_mod_folder.to_path_buf()))?;
	let mod_folder_parent = sevzipped_mod_folder.parent()
		.ok_or(InstallationError::ParentlessArchiveError(sevzipped_mod_folder.to_path_buf()))?;
	let extracted_folder = mod_folder_parent
        .join(folder_name);
	
	decompress_file(sevzipped_mod_folder, mod_folder_parent)?;
	
	Ok(extracted_folder)
}

fn decompress_rar(rared_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
	let folder_name = rared_mod_folder.file_stem()
        .ok_or(InstallationError::NamelessArchiveError(rared_mod_folder.to_path_buf()))?;
	let mod_folder_parent = rared_mod_folder.parent()
		.ok_or(InstallationError::ParentlessArchiveError(rared_mod_folder.to_path_buf()))?;
	let extracted_folder = mod_folder_parent
        .join(folder_name);
	
	let mut rar_archive = Archive::new(rared_mod_folder).open_for_processing()?;
	
	while let Some(header) = rar_archive.read_header()? {
  		rar_archive = if header.entry().is_file() {
        	header.extract_to(mod_folder_parent)?
    	} else {
        	header.skip()?
     	};
	}
	
    Ok(extracted_folder)
}



pub fn check_mod_type(mod_folder_path: &mut Path) -> Result<Option<(ModType, PathBuf)>, InstallationError> {
    // Define variables that will be returned
    let mut mod_files_path: Option<PathBuf> = None;
    let mut mod_contained: Option<ModType> = None;

    // Start looking at the contents of mod folder
    for entry in WalkDir::new(&mod_folder_path) {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        // Skip folders
        if !current_entry.file_type().is_file() {
           	continue
        }
        // Get current entry file extension
        let extension = match get_file_extension(entry_path) {
            Ok(ext) => ext,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };

        // For each valid entry check if it is the file of a mod
        mod_contained = match extension {
            "dss" => Some(ModType::Textures),
            "dtt" | "dat" => {
                let Some(name) = entry_path.file_name() else {
                    println!("\"{:?}\" is a path that ends in .. (parent directory) or . (current directory), and will therefore be skipped", entry_path);
                    continue;
                };
                match name.to_str() {
                    Some("pl") => Some(ModType::PlayerModels),
                    Some("wp") => Some(ModType::WeaponModels),
                    Some("bg") => Some(ModType::WorldModels),
                    Some(_) => None,
                    None => {
                        println!("\"{:?}\" contains invalid Unicode in its name and will therefore will be skipped. Ensure this isn't supposed to be a mod file, if it is, than mod may not work without this file", entry_path);
                        continue;
                    }
                }
            }  // RESHADE
            "usm" => Some(ModType::CutsceneReplacements),
            _ => None,
        };

        if mod_contained.is_some() {
            // Update mod_files_path
            mod_files_path = Some(entry_path.to_path_buf());
            break;
        }
    }

    Ok(mod_contained.zip(mod_files_path))
}



/* ------------------------ */
/*   INSTALLATION METHODS   */
/* ------------------------ */

pub fn install_texture(dss_folder_path: &Path, game_path: &Path) -> Result<Mod, InstallationError> {
    let answer = ask_mod_name()?;

    let texture_mods_folder = game_path.join("/SK_Res/inject/textures");

    let mut mod_files: Vec<PathBuf> = vec![];
    for entry in read_dir(dss_folder_path).map_err(|err| InstallationError::DirectoryReadingError(dss_folder_path.to_path_buf(), err))? {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        copy(&entry_path, &texture_mods_folder)
            .map_err(|err| InstallationError::FileCopyingError(entry_path.clone(), texture_mods_folder.clone(), err))?;

        mod_files.push(entry_path);
    }

    Ok(Mod::new(
        answer,
        mod_files,
        true,
        ModType::Textures,
    ))
}

pub fn install_player_model(dtt_dat_folder_path: &Path, game_path: &Path) -> Result<Mod, InstallationError>  {
    let answer = ask_mod_name()?;

    let pl_mods_folder = game_path.join("/data/pl");

    let mut mod_files: Vec<PathBuf> = vec![];
    for entry in read_dir(dtt_dat_folder_path).map_err(|err| InstallationError::DirectoryReadingError(dtt_dat_folder_path.to_path_buf(), err))?  {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        copy(&entry_path, &pl_mods_folder)
            .map_err(|err| InstallationError::FileCopyingError(entry_path.clone(), pl_mods_folder.clone(), err))?;

        mod_files.push(entry_path);
    }

    Ok(Mod::new(
        answer,
        mod_files,
        true,
        ModType::PlayerModels,
    ))
}

pub fn install_weapon_model(dtt_dat_folder_path: &Path, game_path: &Path) -> Result<Mod, InstallationError> {
    let answer = ask_mod_name()?;

    let wp_mods_folder = game_path.join("/data/wp");

    let mut mod_files: Vec<PathBuf> = vec![];
    for entry in read_dir(dtt_dat_folder_path).map_err(|err| InstallationError::DirectoryReadingError(dtt_dat_folder_path.to_path_buf(), err))?  {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        copy(&entry_path, &wp_mods_folder)
            .map_err(|err| InstallationError::FileCopyingError(entry_path.clone(), wp_mods_folder.clone(), err))?;

        mod_files.push(entry_path);
    }

    Ok(Mod::new(
        answer,
        mod_files,
        true,
        ModType::WeaponModels,
    ))
}

pub fn install_world_model(dtt_dat_folder_path: &Path, game_path: &Path) -> Result<Mod, InstallationError> {
    let answer = ask_mod_name()?;

    let bg_mods_folder = game_path.join("/data/bg");

    let mut mod_files: Vec<PathBuf> = vec![];
    for entry in read_dir(dtt_dat_folder_path).map_err(|err| InstallationError::DirectoryReadingError(dtt_dat_folder_path.to_path_buf(), err))?  {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        copy(&entry_path, &bg_mods_folder)
            .map_err(|err| InstallationError::FileCopyingError(entry_path.clone(), bg_mods_folder.clone(), err))?;

        mod_files.push(entry_path);
    }

    Ok(Mod::new(
        answer,
        mod_files,
        true,
        ModType::WorldModels,
    ))
}

pub fn install_cutscene_replacements(usm_folder_path: &Path, game_path: &Path) -> Result<Mod, InstallationError> {
    let answer = ask_mod_name()?;

    let cutscene_mods_folder = game_path.join("/data/movie");

    let mut mod_files: Vec<PathBuf> = vec![];
    for entry in read_dir(usm_folder_path).map_err(|err| InstallationError::DirectoryReadingError(usm_folder_path.to_path_buf(), err))?  {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        copy(&entry_path, &cutscene_mods_folder)
            .map_err(|err| InstallationError::FileCopyingError(entry_path.clone(), cutscene_mods_folder.clone(), err))?;

        mod_files.push(entry_path);
    }

    Ok(Mod::new(
        answer,
        mod_files,
        true,
        ModType::CutsceneReplacements,
    ))
}

pub fn install_reshade_preset(preset_folder_path: &Path, game_path: &Path) -> Result<Mod, InstallationError> {
	Ok(Mod::new(String::from("Texture"), vec![], true, ModType::Textures))
}