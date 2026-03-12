use std::path::PathBuf;

use thiserror::Error;



#[derive(Error, Debug)]
pub enum EnablingDisablingError {
    #[error("{0} ends with ..")]
    DotDotPath(PathBuf),
    
    #[error("{0} is either root or an empty path")]
    ParentlessOrEmptyPath(PathBuf),
    
    #[error("")]
    Renaming(#[from] std::io::Error)
}