use std::fs::rename;

use std::path::PathBuf;

use thiserror::Error;

use crate::data_config::{Config, Mod};



#[derive(Error, Debug)]
pub enum EnablingDisablingError {
	#[error("No installed mod has the name {0}")]
	ModNotFound(String),
	
    #[error("{0} ends with ..")]
    DotDotPath(PathBuf),
    
    #[error("{0} is either root or an empty path")]
    ParentlessOrEmptyPath(PathBuf),
    
    #[error("")]
    Renaming(#[from] std::io::Error)
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

pub fn enable_mod(config: &mut Config, mod_name: String) -> Result<(), EnablingDisablingError>  {
	let Some(mod_to_enable) = config.get_mod_by_name(&mod_name) else {
		return Err(EnablingDisablingError::ModNotFound(mod_name))
	};
	
    for file in &mod_to_enable.files {
    	let Some(filename) = file.file_name() else {
     		return Err(EnablingDisablingError::DotDotPath(file.to_path_buf()))
     	};
     	let Some(parent) = file.parent() else {
      		return Err(EnablingDisablingError::ParentlessOrEmptyPath(file.to_path_buf()))
      	};
      	let Some(enabled_folder) = parent.parent() else {
       		return Err(EnablingDisablingError::ParentlessOrEmptyPath(parent.to_path_buf()))
       	};
       
      	rename(file, enabled_folder.join(filename));
    }
    
    Ok(())
}

pub fn disable_mod(config: &mut Config, mod_name: String) -> Result<(), EnablingDisablingError>  {
	let Some(mod_to_disable) = config.get_mod_by_name(&mod_name) else {
		return Err(EnablingDisablingError::ModNotFound(mod_name))
	};
	
    for file in &mod_to_disable.files {
    	let Some(filename) = file.file_name() else {
     		return Err(EnablingDisablingError::DotDotPath(file.to_path_buf()))
     	};
     	let Some(parent) = file.parent() else {
      		return Err(EnablingDisablingError::ParentlessOrEmptyPath(file.to_path_buf()))
      	};
      	let disabled_folder = parent.join(".disabled/");
       
      	rename(file, disabled_folder.join(filename));
    }
    
    Ok(())
}