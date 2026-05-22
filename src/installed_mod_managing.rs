use std::fs::{create_dir_all, rename};

use std::path::PathBuf;

use thiserror::Error;

use crate::data_config::{Config, Mod, ConfigInteractionError};



#[derive(Error, Debug)]
pub enum EnablingDisablingError {
	#[error("No installed mod has the name {0}")]
	ModNotFound(String),
	
    #[error("{0} ends with ..")]
    DotDotPath(PathBuf),
    
    #[error("{0} is either root or an empty path")]
    ParentlessOrEmptyPath(PathBuf),
    
    #[error("Couldn't create {0}. {1}")]
    FolderCreation(PathBuf, std::io::Error),
    
    #[error("Couldn't move file from downloaded folder to game folder. {0}")]
    Renaming(#[from] std::io::Error),
    
    #[error("Couldn't update data file (~/.config/ATA/data.json) to add newly installed mod")]
    DataSaving(#[from] ConfigInteractionError)
}



pub fn list_mods(mods: &Vec<Mod>) {
    println!("List of mods:\n");
    for installed_mod in mods {
        println!("\t- {}\n", installed_mod.name);
        
        println!("\t\tFiles: ");
        for file in installed_mod.files.clone() {
            println!("\t\t- {:?}", file)
        }
        
        println!("Enabled: {}", if installed_mod.enabled==true {"Yes"} else {"No"});
        
        println!("Mod Type: {}", installed_mod.mod_type)
    }
}

pub fn enable_mod(config: &mut Config, mod_name: String) -> Result<Mod, EnablingDisablingError>  {
	let Some(mut mod_to_enable) = config.get_mod_by_name(&mod_name) else {
		return Err(EnablingDisablingError::ModNotFound(mod_name))
	};
	let mut updated_files: Vec<PathBuf> = vec![];
	
    for file in &mod_to_enable.1.files {
    	let Some(filename) = file.file_name() else {
     		return Err(EnablingDisablingError::DotDotPath(file.to_path_buf()))
     	};
     	let Some(parent) = file.parent() else {
      		return Err(EnablingDisablingError::ParentlessOrEmptyPath(file.to_path_buf()))
      	};
      	let Some(enabled_folder) = parent.parent() else {
       		return Err(EnablingDisablingError::ParentlessOrEmptyPath(parent.to_path_buf()))
       	};
        
        let new_path = enabled_folder.join(filename);
      	rename(file, &new_path)?;
        updated_files.push(new_path);
    }
    
    mod_to_enable.1.enabled = true;
    mod_to_enable.1.files = updated_files;
    
    Ok(mod_to_enable.1.clone())
}

pub fn disable_mod(config: &mut Config, mod_name: String) -> Result<Mod, EnablingDisablingError>  {
	let Some(mut mod_to_disable) = config.get_mod_by_name(&mod_name) else {
		return Err(EnablingDisablingError::ModNotFound(mod_name))
	};
	let mut updated_files: Vec<PathBuf> = vec![];
	
    for file in &mod_to_disable.1.files {
    	let Some(filename) = file.file_name() else {
     		return Err(EnablingDisablingError::DotDotPath(file.to_path_buf()))
     	};
     	let Some(parent) = file.parent() else {
      		return Err(EnablingDisablingError::ParentlessOrEmptyPath(file.to_path_buf()))
      	};
      	let disabled_folder = parent.join(".disabled/");
       
        create_dir_all(&disabled_folder)
            .map_err(|er| EnablingDisablingError::FolderCreation(disabled_folder.to_path_buf(), er))?;
       
      	rename(file, disabled_folder.join(filename))?;
       
        updated_files.push(disabled_folder.join(filename));
    }
    
    mod_to_disable.1.enabled = false;
    mod_to_disable.1.files = updated_files;
    
    Ok(mod_to_disable.1.clone())
}