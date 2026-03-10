use std::fs::remove_file;

use std::io::{stdin, stdout, Write};

use std::path::PathBuf;

use thiserror::Error;



#[derive(Error, Debug)]
pub enum UninstallationError {
    #[error("Encountered an error while trying to read/write the console. {0}")]
    ConsoleInteraction(#[from] std::io::Error),
    
    #[error("No mod named '{0}' found")]
    ModNotFound(String),
    
    #[error("Couldn't remove {0} from the game's directory")]
    FileDeletion(PathBuf, std::io::Error)
}

pub fn ask_for_mod_name() -> Result<String, UninstallationError> {
    println!("Enter the name of the mod to uninstall:");
    print!("Mod name>> ");
    stdout().flush()?;
    
    let mut mod_name = String::new();
    stdin().read_line(&mut mod_name)?;
    
    Ok(mod_name)
}

pub fn remove_mod_files(mod_files: Vec<PathBuf>) -> Result<(), UninstallationError> {
    for file in mod_files {
        remove_file(&file)
            .map_err(|er| UninstallationError::FileDeletion(file.clone(), er))?;
    }

    Ok(())
}