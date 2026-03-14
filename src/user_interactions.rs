use std::process::Command;
use std::io::stdin;
use std::io::stdout;

use thiserror::Error;



#[derive(Error, Debug)]
pub enum CommandError {
    #[error("{0} was killed with a system signal")]
    SignalKill(String),
    
    #[error("{0} crashed with exit code {1}")]
    CommandCrash(String, i32),

    #[error("{0} process failed to spawn")]
    FailedSpawn(#[from] std::io::Error)
}



pub fn ask_user_action() -> Result<String, CommandError> {
    let mut child = Command::new("fzf")
        .arg("--prompt=What do you want to do? ")
        .arg("--height=10%")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if !fzf_status.success() {
        return match fzf_status.code() {
            Some(code) => Err(CommandError::CommandCrash("fzf".to_string(), code)),
            None =>       Err(CommandError::SignalKill("fzf".to_string())),
        };
    }
    
    return Ok(String::new())
}