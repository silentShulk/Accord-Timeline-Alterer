/// *installation* is a module that contains the functions needed to installa a mod
/// 
/// This includes:
/// * **decompressing**: Going from the compressed archive to a normal folder
/// * **understanding the mod**: Looks at the type of files in the folder to understand what type of mod it is
/// * **installing the mod**: Moves the files found to be of the mod in the folder in which that mod type goes
/// * **updates the saved data**: Adds the newly installed mod to the data file (*~/.config/ATA/data.json*) 



use std::fs::{File, copy, create_dir_all, remove_file};

use std::path::{PathBuf, Path};

use thiserror::Error;

use walkdir::WalkDir;

use zip::ZipArchive;

use sevenz_rust::decompress_file;

use unrar::Archive;

use crate::data_config::{Config, ConfigInteractionError, Mod, ModType};



/// Installs a mod found in a compressed archive and saves it in a config with user-decided name
/// 
/// # Returns
/// The Mod installed
/// 
/// # Errors
/// * [`InstallationError::FileAccessing`] if the path to the compressed archive leads to nothing
/// * [`InstallationError::`]
pub fn install_mod(compressed_mod_folder_path: &Path, config: &mut Config, answered_name: String) -> Result<Mod, InstallationError> {
    // Check if it exists
    if !compressed_mod_folder_path.exists() {
        return Err(InstallationError::FileNotFound(compressed_mod_folder_path.to_path_buf()));
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
    
    #[error("{0} is a parentless file")]
    ParentlessFile(PathBuf),
    
    #[error("{0} is of an unsupported compression type (supported types are .zip, .7z .rar)")]
    UnsupportedCompression(PathBuf),
    
    #[error("Couldn't access/open {0}")]
    FileAccessing(#[from] std::io::Error),
    
    #[error("Couldn't extract zip file {0}")]
    ZipExtracionFailed(#[from] zip::result::ZipError),      // Zip specific
    
    #[error("Couldn't extract 7z file {0}")]
    SevenZipExtractionFailed(#[from] sevenz_rust::Error),   // 7z specific
    
    #[error("Couldn't extract rar file {0}")]
    RarExtractionFailed(#[from] unrar::error::UnrarError),  // Rar specific
    
    // Get mod data
    #[error("Couldn't read entry from mod folder. {0}")]
    EntryReading(#[from] walkdir::Error),
    
    #[error("The given folder doesn't contain a mod")]
    ModlessFolder(PathBuf),
    
    // File copying
    #[error("Couldn't create {0}. {1}")]
    FolderCreation(PathBuf, std::io::Error),
    
    #[error("{0} is either root or ends in ..")]
    InvalidFileName(PathBuf),
    
    #[error("Couldn't copy {0} to {1}. {2}")]
    FileCopying(PathBuf, PathBuf, std::io::Error),
    
    // Data saving
    #[error("Couldn't update data file (~/.config/ATA/data.json). {0}")]
    DataSaving(#[from] ConfigInteractionError),
}



pub fn decompress_folder(compressed_mod_folder: &Path) -> Result<PathBuf, InstallationError> {
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
    let zip_file = File::open(zipped_mod_folder)?;
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
        	remove_file(entry_path)?;
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
        		return Err(InstallationError::InvalidFileName(file.to_path_buf()))
        };
        
        let copied_file = destination_folder_path.join(filename); 
        copy(file, &copied_file)
            .map_err(|er| InstallationError::FileCopying(file.as_path().to_path_buf(), destination_folder_path.clone(), er))?;
        
        copied_files.push(copied_file);
    }

    Ok(copied_files)
}



pub fn install(mod_type: &ModType, mod_files: &Vec<PathBuf>, game_path: &PathBuf) -> Result<Vec<PathBuf>, InstallationError> {
    let installation_folder = game_path.join(mod_type.get_corresponding_folder());
    
    let installed_files = copy_mod_files(mod_files, PathBuf::from(installation_folder))?;
    
    Ok(installed_files)
}