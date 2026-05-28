use std::fs::{create_dir_all, rename};

use std::path::PathBuf;

use thiserror::Error;

use crate::data::{Data, Mod, DataInteractionError};



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
    
    #[error("Couldn't update data file (data.json found inside data dir of OS). {0}")]
    DataSaving(#[from] DataInteractionError)
}



pub fn list_mods(data: &Data) -> Vec<Mod> {
	data.mods.clone()
}

pub fn enable_mod(data: &mut Data, mod_name: String) -> Result<Mod, EnablingDisablingError>  {
	let Some(mod_to_enable) = data.get_mod_by_name(&mod_name) else {
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

    data.switch_mod_state(mod_to_enable.0, updated_files)?;
    
    Ok(data.mods[mod_to_enable.0].clone())
}

pub fn disable_mod(data: &mut Data, mod_name: String) -> Result<Mod, EnablingDisablingError>  {
	let Some(mod_to_disable) = data.get_mod_by_name(&mod_name) else {
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
    
    data.switch_mod_state(mod_to_disable.0, updated_files)?;
    
    Ok(data.mods[mod_to_disable.0].clone())
}