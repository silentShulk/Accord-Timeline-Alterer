use std::io::{stdin, stdout, Write};

use thiserror::Error;



#[derive(Error, Debug)]
pub enum UninstallationError {
    // Console interaction
    #[error("Encountered an error while trying to read/write the console. {0}")]
    ConsoleInteraction(#[from] std::io::Error),
    
    #[error("No mod named '{0}' found")]
    ModNotFound(String),
}

pub fn ask_for_mod_name() -> Result<String, UninstallationError> {
    println!("Enter the name of the mod to uninstall:");
    print!("Mod name>> ");
    stdout().flush()?;
    
    let mut mod_name = String::new();
    stdin().read_line(&mut mod_name)?;
    
    Ok(mod_name)
}