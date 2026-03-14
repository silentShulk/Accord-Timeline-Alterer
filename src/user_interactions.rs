use std::{io::Write, process::{Command, Stdio}, string::FromUtf8Error};

use thiserror::Error;



#[derive(Error, Debug)]
pub enum CommandError {
    #[error("{0} process failed to spawn")]
    FailedSpawn(std::io::Error),
    
    #[error("{0} has no pipe handle")]
    MissingPipeHandle(String),
    
    #[error("{0} occured while writing {1} to the buffer of stdin")]
    BufferWriting(std::io::Error, String),
    
    #[error("A critical error occured while trying to read {0} output. {1}")]
    ProcessOutputReading(String, std::io::Error),
    
    #[error("{0} was killed with a system signal")]
    SignalKill(String),
    
    #[error("{0} crashed with exit code {1}")]
    CommandCrash(String, i32),
    
    #[error("Selection contained invalid UTF-8. {0}")]
    InvalidUTF8InSelection(#[from] FromUtf8Error)
}



pub fn ask_user_action() -> Result<String, CommandError> {
    let mut fzf_child = Command::new("fzf")
        .arg("--prompt=What do you want to do? ")
        .arg("--height=10%")
        .arg("--reverse")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|er| CommandError::FailedSpawn(er))?;
    
    let options = "Install a mod\nUninstall a mod\nList mods\nEnable a mod\nDisable a mod";
    
    let mut fzf_stdin = fzf_child.stdin.take()
        .ok_or(CommandError::MissingPipeHandle("fzf".to_string()))?;
    fzf_stdin.write_all(options.as_bytes())
        .map_err(|er| CommandError::BufferWriting(er, options.to_string()))?;
    
    let fzf_output = fzf_child.wait_with_output()
        .map_err(|er| CommandError::ProcessOutputReading("fzf".to_string(), er))?;

    if !fzf_output.status.success() {
        if let Some(code) = fzf_output.status.code() {
            return Err(CommandError::CommandCrash("fzf".to_string(), code))
        } else {
            return Err(CommandError::SignalKill("fzf".to_string()))
        }
    }

    let user_selection = String::from_utf8(fzf_output.stdout)?;
    
    return Ok(user_selection)
}