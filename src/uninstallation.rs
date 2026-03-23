use std::fs::remove_file;

use std::path::PathBuf;

use thiserror::Error;

use crate::data_config::Config;



pub fn uninstall_mod(config: &mut Config, mod_name: String) -> Result<usize, UninstallationError> {
    let index_to_uninstall = config.mods
        .iter()
        .position(|m| m.name == mod_name)
        .ok_or(UninstallationError::ModNotFound(mod_name))?;

    remove_mod_files(config.mods[index_to_uninstall].clone().files)?;
    
    Ok(index_to_uninstall)
}



#[derive(Error, Debug)]
pub enum UninstallationError {
    #[error("Encountered an error while trying to read/write the console. {0}")]
    ConsoleInteraction(#[from] std::io::Error),
    
    #[error("No mod named '{0}' found")]
    ModNotFound(String),
    
    #[error("Couldn't remove {0} from the game's directory")]
    FileDeletion(PathBuf, std::io::Error)
}



pub fn remove_mod_files(mod_files: Vec<PathBuf>) -> Result<(), UninstallationError> {
    for file in mod_files {
        remove_file(&file)
            .map_err(|er| UninstallationError::FileDeletion(file.clone(), er))?;
    }

    Ok(())
}