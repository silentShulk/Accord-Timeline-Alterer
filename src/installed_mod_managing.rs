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



pub fn enable_mod(config: &mut Config, mod_name: String) -> Result<Mod, EnablingDisablingError>  {
	let Some(mod_to_enable) = config.get_mod_by_name(&mod_name) else {
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

    config.switch_mod_state(mod_to_enable.0, updated_files)?;
    
    Ok(config.mods[mod_to_enable.0].clone())
}

pub fn disable_mod(config: &mut Config, mod_name: String) -> Result<Mod, EnablingDisablingError>  {
	let Some(mod_to_disable) = config.get_mod_by_name(&mod_name) else {
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
    
    config.switch_mod_state(mod_to_disable.0, updated_files)?;
    
    Ok(config.mods[mod_to_disable.0].clone())
}