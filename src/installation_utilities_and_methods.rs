use std::fs::{copy, read_dir, File};

use std::io::{stdin, stdout, Write};

use std::path::{PathBuf, Path};

use thiserror::Error;

use walkdir::WalkDir;

use zip::ZipArchive;

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

    #[error("Found a file with an extension containing invalid UTF-8, it will be skipped")]
    InvalidExtensionError(PathBuf),
    
    // Decompression
    #[error("The received compressed folder uses an unsupported extension")]
    UnsupportedCompressionError(String),
    
    #[error("Couldn't access a file")]
    FileAccessingError,
    
    #[error("Couldn't extract a compressed folder: {0}")]
    FailedExtractionError(),

    // Directory Reading Errors
    #[error("Couldn't find or read completely a directory")]
    DirectoryReadingError,
    
    #[error("Couldn't read an entry inside a folder")]
    EntryReadingError(),
    
    #[error("Couldn't copy file")]
    FileCopyingError,

    #[error("Encountered an error while trying to read/write the console")]
    ConsoleInteractionError(),
}


pub fn decompress_folder(compressed_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
    let extension = get_file_extension(compressed_mod_folder)?;
    
    match extension {
        ".zip" => Ok(decompress_zip(compressed_mod_folder)?),
        ".rar" => Ok(decompress_rar(compressed_mod_folder)?),
        ".7z" => Ok(decompress_7z(compressed_mod_folder)?),
        _ => Err(InstallationError::UnsupportedCompressionError(extension.to_string()))
    }
}

fn decompress_zip(zipped_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
    let zip_file = File::open(zipped_mod_folder)?;
    let zip_archive = ZipArchive::new(zip_file)?;
    
    
    
    Ok(PathBuf::new())
}
fn decompress_7z(sevzipped_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
    Ok(PathBuf::new())
}
fn decompress_rar(rared_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
    Ok(PathBuf::new())
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
                        println!("\"{:?}\" contains invalid Unicode in its name and will therefore will be skipped", entry_path);
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

fn get_file_extension(path: &Path) -> Result<&str, InstallationError> {
    let Some(extension) = path.extension() else {
        return Err(InstallationError::ExtensionlessFileError(path.to_path_buf()));
    };
    let Some(extension_str) = extension.to_str() else {
       	return Err(InstallationError::InvalidExtensionError(path.to_path_buf()));
    };

    Ok(extension_str)
}



/* ------------------------ */
/*   INSTALLATION METHODS   */
/* ------------------------ */

pub fn install_texture(dss_folder_path: &Path, game_path: &Path) -> Result<Mod, InstallationError> {
    let answer = ask_mod_name()?;

    let texture_mods_folder = game_path.join("/SK_Res/inject/textures");

    let mut mod_files: Vec<PathBuf> = vec![];
    for entry in read_dir(dss_folder_path)? {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        copy(&entry_path, &texture_mods_folder)?;

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
    for entry in read_dir(dtt_dat_folder_path)? {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        copy(&entry_path, &pl_mods_folder)?;

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
    for entry in read_dir(dtt_dat_folder_path)? {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        copy(&entry_path, &wp_mods_folder)?;

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
    for entry in read_dir(dtt_dat_folder_path)? {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        copy(&entry_path, &bg_mods_folder)?;

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
    for entry in read_dir(usm_folder_path)? {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        copy(&entry_path, &cutscene_mods_folder)?;

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