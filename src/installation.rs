use std::fs::{copy, read_dir, File};

use std::io::{stdin, stdout, Write};

use std::path::{PathBuf, Path};

use thiserror::Error;

use walkdir::WalkDir;

use zip::ZipArchive;

use sevenz_rust::decompress_file;

use unrar::Archive;

use crate::data_config::{ModType, Mod, Config};



pub fn install_mod(config: &mut Config, compressed_mod_folder_path: &Path) -> Result<Mod, InstallationError> {
    // Check if it exists
    if !compressed_mod_folder_path.exists() {
        return Err(InstallationError::FileAccessing(compressed_mod_folder_path.to_path_buf()));
    }
    
    // Ask for a name 
    let answered_name = ask_mod_name()?;
    
    // Unzip the mod folder
    let mut mod_folder_path = decompress_folder(&compressed_mod_folder_path)?;
    
    // Get the type of mod contained
    let mod_data = get_mod_data(&mut mod_folder_path)?
       	.ok_or(InstallationError::ModlessFolder(mod_folder_path.clone()))?;
    
    // Install the mod contained in the folder following the correct installation method
    let installed_mod_files = match mod_data.0 {
       	ModType::Textures => install_texture(mod_folder_path, &config.game_path)?,
       	ModType::PlayerModels => install_player_model(mod_folder_path, &config.game_path)?,
       	ModType::WeaponModels => install_weapon_model(mod_folder_path, &config.game_path)?,
       	ModType::WorldModels => install_world_model(mod_folder_path, &config.game_path)?,
        ModType::CutsceneReplacements => install_cutscene_replacements(mod_folder_path, &config.game_path)?,
        ModType::ReshadePreset => install_reshade_preset(&mod_folder_path, &config.game_path)?,
    };
    let installed_mod = Mod::new(answered_name, installed_mod_files, true, mod_data.0);
    
    // Updates config
   	config.save_new_mod(&installed_mod).unwrap_or_else(|er| {
  		eprintln!("Therer was a problem adding the newly installed mod to the data file. {}
				ATA will now close...", er);
        std::process::exit(1);
   	});
    
    Ok(installed_mod)
}



/* ------------- */
/*   UTILITIES   */
/* ------------- */

#[derive(Error, Debug)]
pub enum InstallationError {
    // Console interaction
    #[error("Couldn't flush stdout. {0}")]
    StdoutFlush(std::io::Error),
    
    #[error("Couldn't read from stdin. {0}")]
    StdinRead(std::io::Error),
    
    // Extension Reading
    #[error("{0} is an an extensionless file, it will be skipped")]
    ExtensionlessFile(PathBuf),

    #[error("{0} has an extension containing invalid UTF-8, it will be skipped")]
    InvalidExtension(PathBuf),
    
    // Decompression
    #[error("The received compressed folder ({0}) uses an unsupported extension")]
    UnsupportedCompression(PathBuf),
    
    #[error("Couldn't access {0}, check if it exists")]
    FileAccessing(PathBuf),
    
    #[error("Couldn't extract zip archive. {0}")]
    FailedZipExtraction(#[from] zip::result::ZipError),
    
    #[error("Compressed archive ({0}) doesn't have a parent directory")]
    ParentlessArchive(PathBuf),
    
    #[error("The compressed archive ({0}) doesn't have a name")]
    NamelessArchive(PathBuf),
       
    #[error("Couldn't extract 7z archive. {0}")]
    Failed7zExtraction(#[from] sevenz_rust::Error),
    
    #[error("Couldn't extract rar archive. {0}")]
    FailedRarExtraction(#[from] unrar::error::UnrarError),

    // Directory Reading Errors
    #[error("Couldn't read an entry. {0}")]
    ModFolderEntryReading(#[from] walkdir::Error),
    
    #[error("Couldn't read the mod folder. {0}")]
    ModFileReading(#[from] std::io::Error),
    
    #[error("Couldn't find or read completely {0}")]
    DirectoryReading(PathBuf, std::io::Error),

    #[error("Couldn't copy {0} to {1}. {2}")]
    FileCopying(PathBuf, PathBuf, std::io::Error),
    
    #[error("The given folder doesn't contain a mod")]
    ModlessFolder(PathBuf),
}



// DECOMPRESSING THE MOD FOLDER
pub fn decompress_folder(compressed_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
    let extension = get_file_extension(compressed_mod_folder)?;
   
   	let folder_name = compressed_mod_folder.file_stem()
           .ok_or(InstallationError::NamelessArchive(compressed_mod_folder.to_path_buf()))?;
	let mod_folder_parent = compressed_mod_folder.parent()
		.ok_or(InstallationError::ParentlessArchive(compressed_mod_folder.to_path_buf()))?;
    let target_folder = mod_folder_parent
           .join(folder_name);
    
    match extension {
        "zip" => decompress_zip(compressed_mod_folder, target_folder),
        "7z" => decompress_7z(compressed_mod_folder, target_folder),
        "rar" => decompress_rar(compressed_mod_folder, target_folder),
        _ => Err(InstallationError::UnsupportedCompression(compressed_mod_folder.to_path_buf())),
    }
}

fn get_file_extension(path: &Path) -> Result<&str, InstallationError> {
    let Some(extension) = path.extension() else {
        return Err(InstallationError::ExtensionlessFile(path.to_path_buf()));
    };
    let Some(extension_str) = extension.to_str() else {
       	return Err(InstallationError::InvalidExtension(path.to_path_buf()));
    };

    Ok(extension_str)
}

fn decompress_zip(zipped_mod_folder: &Path, zip_extraction_folder: PathBuf) -> Result<PathBuf, InstallationError> {
    let zip_file = File::open(zipped_mod_folder)
        .map_err(|_| InstallationError::FileAccessing(zipped_mod_folder.to_path_buf()))?;
    let mut zip_archive = ZipArchive::new(zip_file)?;
    
    zip_archive.extract(&zip_extraction_folder)?;
    
    Ok(zip_extraction_folder)
}

fn decompress_7z(sevzipped_mod_folder: &Path, sevzip_extraction_folder: PathBuf) -> Result<PathBuf, InstallationError> {
	decompress_file(sevzipped_mod_folder, &sevzip_extraction_folder)?;
	
	Ok(sevzip_extraction_folder)
}

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



pub fn ask_mod_name() -> Result<String, InstallationError> {
	println!("Insert name of the mod that you are installing (choose anything you want, will be used as identifier)");
	print!("Name: ");
	stdout().flush().map_err(|er| InstallationError::StdoutFlush(er))?;

	let mut answer = String::new();
	stdin().read_line(&mut answer).map_err(|er| InstallationError::StdinRead(er))?;
	Ok(answer)
}



// UNDERSTANDING MOD TYPE AND FILTERING OUT UNNECESSARY FILES
pub fn get_mod_data(mod_folder_path: &mut Path) -> Result<Option<(ModType, PathBuf)>, InstallationError> {
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



fn copy_mod_files(mod_files_path: PathBuf, destination_folder_path: PathBuf) -> Result<Vec<PathBuf>, InstallationError> {
    let mut copied_files: Vec<PathBuf> = Vec::new();
    let aura = read_dir(&mod_files_path).map_err(|err| InstallationError::DirectoryReading(mod_files_path.to_path_buf(), err))?;
    
    for entry in aura {
        let current_entry = entry?;
        let entry_path = current_entry.path();

        copy(&entry_path, &destination_folder_path)
            .map_err(|err| InstallationError::FileCopying(entry_path.clone(), destination_folder_path.clone(), err))?;
        
        copied_files.push(entry_path);
    }
    
    Ok(copied_files)
}



/* ------------------------ */
/*   INSTALLATION METHODS   */
/* ------------------------ */

pub fn install_texture(dss_folder_path: PathBuf, game_path: &Path) -> Result<Vec<PathBuf>, InstallationError> {
    let texture_mods_folder = game_path.join("/SK_Res/inject/textures");

    let mod_files = copy_mod_files(dss_folder_path, texture_mods_folder)?;
    
    Ok(mod_files)
}

pub fn install_player_model(dtt_dat_folder_path: PathBuf, game_path: &Path) -> Result<Vec<PathBuf>, InstallationError>  {
    let pl_mods_folder = game_path.join("/data/pl");

    let mod_files = copy_mod_files(dtt_dat_folder_path, pl_mods_folder)?;

    Ok(mod_files)
}

pub fn install_weapon_model(dtt_dat_folder_path: PathBuf, game_path: &Path) -> Result<Vec<PathBuf>, InstallationError> {
    let wp_mods_folder = game_path.join("/data/wp");

    let mod_files = copy_mod_files(dtt_dat_folder_path, wp_mods_folder)?;
    
    Ok(mod_files)
}

pub fn install_world_model(dtt_dat_folder_path: PathBuf, game_path: &Path) -> Result<Vec<PathBuf>, InstallationError> {
    let bg_mods_folder = game_path.join("/data/bg");

    let mod_files = copy_mod_files(dtt_dat_folder_path, bg_mods_folder)?;

    Ok(mod_files)
}

pub fn install_cutscene_replacements(usm_folder_path: PathBuf, game_path: &Path) -> Result<Vec<PathBuf>, InstallationError> {
    let cutscene_mods_folder = game_path.join("/data/movie");

    let mod_files = copy_mod_files(usm_folder_path, cutscene_mods_folder)?;

    Ok(mod_files)
}

pub fn install_reshade_preset(preset_folder_path: &Path, game_path: &Path) -> Result<Vec<PathBuf>, InstallationError> {
    Ok(Vec::new())
}