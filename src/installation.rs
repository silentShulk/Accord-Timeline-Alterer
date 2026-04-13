use std::fs::{File, copy, create_dir_all, remove_file};

use std::path::{PathBuf, Path};

use thiserror::Error;

use walkdir::WalkDir;

use zip::ZipArchive;

use sevenz_rust::decompress_file;

use unrar::Archive;

use crate::data_config::{Config, ConfigInteractionError, Mod, ModType};



pub fn install_mod(config: &mut Config, compressed_mod_folder_path: &Path, answered_name: String) -> Result<Mod, InstallationError> {
    // Check if it exists
    if !compressed_mod_folder_path.exists() {
        return Err(InstallationError::FileAccessing(compressed_mod_folder_path.to_path_buf()));
    }

    // Unzip the mod folder
    let mut mod_folder_path = decompress_folder(&compressed_mod_folder_path)?;

    // Get the type of mod contained
    let mod_data = get_mod_data(&mut mod_folder_path)?
       	.ok_or(InstallationError::ModlessFolder(mod_folder_path.clone()))?;

    // Install the mod contained in the folder following the correct installation method
    let mod_files =install(&mod_data.0, &mod_data.1, &config.game_path)?;
    
    let installed_mod = Mod::new(answered_name, mod_files, true, mod_data.0);

    // Updates config
   	config.save_new_mod(&installed_mod)
        .map_err(|er| InstallationError::DataSaving(er))?;

    Ok(installed_mod)
}



/* ------------- */
/*   UTILITIES   */
/* ------------- */

#[derive(Error, Debug)]
pub enum InstallationError {
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

    #[error("Couldn't create {0} directory inside of game's folder. {1}")]
    FolderCreation(PathBuf, std::io::Error),
    
    #[error("{0} ends with ..")]
    DotDotPath(PathBuf),

    #[error("Couldn't copy {0} to {1}. {2}")]
    FileCopying(PathBuf, PathBuf, std::io::Error),

    #[error("Couldn't remove a file")]
    FileRemoval(PathBuf, std::io::Error),

    #[error("The given folder doesn't contain a mod")]
    ModlessFolder(PathBuf),
    
    #[error("Couldn't update data file (~/.config/ATA/data.json) to add newly installed mod")]
    DataSaving(#[from] ConfigInteractionError)
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



// UNDERSTANDING MOD TYPE AND FILTERING OUT NON-MOD FILES AND KEEPING FOLDER STRUCTURE
pub fn get_mod_data(mod_folder_path: &Path) -> Result<Option<(ModType, Vec<PathBuf>)>, InstallationError> {
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
        	remove_file(entry_path).map_err(|er| InstallationError::FileRemoval(entry_path.to_path_buf(), er))?;
         	continue;
        }

        mod_files.get_or_insert_with(Vec::new).push(entry_path.to_path_buf());

        if mod_contained.is_none() {
            mod_contained = match extension {
                "dds" => Some(ModType::Textures),
                "dtt" | "dat" => {
                    let name = entry_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                      
                    if name.starts_with("pl") { Some(ModType::PlayerModels) }
                    else if name.starts_with("wp") { Some(ModType::WeaponModels) }
                    else if name.starts_with("bg") { Some(ModType::WorldModels) }
                    else { None }
                }
                "usm" => Some(ModType::CutsceneReplacements),
                _ => None,
            };
        }
    }

    Ok(mod_contained.zip(mod_files))
}



fn copy_mod_files(mod_files: &Vec<PathBuf>, destination_folder_path: PathBuf) -> Result<Vec<PathBuf>, InstallationError> {
    create_dir_all(&destination_folder_path)
        .map_err(|er| InstallationError::FolderCreation(destination_folder_path.clone(), er))?;

    let mut copied_files: Vec<PathBuf> = vec![];
    for file in mod_files {
       	let Some(filename) = file.file_name() else {
        		return Err(InstallationError::DotDotPath(file.to_path_buf()))
        };
        
        let copied_file = destination_folder_path.join(filename); 
        copy(file, &copied_file)
            .map_err(|er| InstallationError::FileCopying(file.as_path().to_path_buf(), destination_folder_path.clone(), er))?;
        
        copied_files.push(copied_file);
    }

    Ok(copied_files)
}



/* ------------------------ */
/*   INSTALLATION METHODS   */
/* ------------------------ */

pub fn install(mod_type: &ModType, mod_files: &Vec<PathBuf>, game_path: &PathBuf) -> Result<Vec<PathBuf>, InstallationError> {
    let installation_folder = game_path.join(mod_type.get_corresponding_folder());
    
    let installed_files = copy_mod_files(mod_files, PathBuf::from(installation_folder))?;
    
    Ok(installed_files)
}