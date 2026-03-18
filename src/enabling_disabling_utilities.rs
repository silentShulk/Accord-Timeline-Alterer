use std::path::PathBuf;

use thiserror::Error;

use crate::data_config::Config;



#[derive(Error, Debug)]
pub enum EnablingDisablingError {
    #[error("{0} ends with ..")]
    DotDotPath(PathBuf),
    
    #[error("{0} is either root or an empty path")]
    ParentlessOrEmptyPath(PathBuf),
    
    #[error("")]
    Renaming(#[from] std::io::Error)
}

pub fn check_if_mod_exists(config: Config, mod_name: String) -> bool {
    let mut exists = false;
    
    for installed_mod in config.mods {
        if installed_mod.name == mod_name {
            exists = true;
        }
    }
    
    exists
}