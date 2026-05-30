use std::fs::remove_file;

use std::path::PathBuf;

use thiserror::Error;

use crate::data::{Data, Mod, DataInteractionError};



pub fn uninstall_mod(config: &mut Data, mod_name: String) -> Result<Mod, UninstallationError> {
    let Some(mod_to_uninstall) = config.get_mod_by_name(&mod_name) else {
        return Err(UninstallationError::ModNotFound(mod_name));
    };

    remove_mod_files(&mod_to_uninstall.1.files)?;

    config.remove_mod(mod_to_uninstall.0)
        .map_err(|e| UninstallationError::DataSaving(e))?;
    
    Ok(mod_to_uninstall.1)
}



#[derive(Error, Debug)]
pub enum UninstallationError {
    #[error("Encountered an error while trying to read/write the console. {0}")]
    FileAccessing(#[from] std::io::Error),
    
    #[error("No mod named '{0}' found")]
    ModNotFound(String),
    
    #[error("Couldn't remove {0} from the game's directory")]
    FileDeletion(PathBuf, std::io::Error),

    #[error("Couldn't update data file (data.json found inside data dir of OS). {0}")]
    DataSaving(#[from] DataInteractionError),
}



pub fn remove_mod_files(mod_files: &Vec<PathBuf>) -> Result<(), UninstallationError> {
    for file in mod_files {
        remove_file(&file)
            .map_err(|er| UninstallationError::FileDeletion(file.clone(), er))?;
    }

    Ok(())
}