use std::path::PathBuf;

use thiserror::Error;

use crate::data_config::Config;



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



