use std::io::{Write, stdout, stdin};

use std::process::{Command, Stdio};

use std::string::FromUtf8Error;

use std::path::PathBuf;

use thiserror::Error;



#[derive(Error, Debug)]
pub enum UserInteractionError {
    #[error("{0} process failed to spawn. {1}")]
    FailedSpawn(String, std::io::Error),

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
    InvalidUTF8InSelection(#[from] FromUtf8Error),

    #[error("Couldn't flush stdout. {0}")]
    StdoutFlush(std::io::Error),

    #[error("Couldn't read from stdin. {0}")]
    StdinRead(std::io::Error),
}



pub fn ask_user_action() -> Result<String, UserInteractionError> {
    let mut fzf_child = Command::new("fzf")
        .arg("--prompt=What do you want to do? ")
        .arg("--height=10%")
        .arg("--reverse")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|er| UserInteractionError::FailedSpawn("fzf".to_string(), er))?;

    let options = "Install a mod\nUninstall a mod\nList mods\nEnable a mod\nDisable a mod\nClose ATA :(";

    let mut fzf_stdin = fzf_child.stdin.take()
        .ok_or(UserInteractionError::MissingPipeHandle("fzf".to_string()))?;
    fzf_stdin.write_all(options.as_bytes())
        .map_err(|er| UserInteractionError::BufferWriting(er, options.to_string()))?;

    let fzf_output = fzf_child.wait_with_output()
        .map_err(|er| UserInteractionError::ProcessOutputReading("fzf".to_string(), er))?;

    if !fzf_output.status.success() {
        if let Some(code) = fzf_output.status.code() {
            return Err(UserInteractionError::CommandCrash("fzf".to_string(), code))
        } else {
            return Err(UserInteractionError::SignalKill("fzf".to_string()))
        }
    }

    let user_selection = String::from_utf8(fzf_output.stdout)?;

    Ok(user_selection.trim().to_string())
}

pub fn ask_for_mod_name() -> Result<String, UserInteractionError> {
    println!("Enter the name of the mod:");
    print!("Mod name>> ");
    stdout().flush().map_err(|er| {
   		return UserInteractionError::StdoutFlush(er);
    })?;

    let mut mod_name = String::new();
    stdin().read_line(&mut mod_name).map_err(|er| {
       		return UserInteractionError::StdinRead(er);
    })?;

    Ok(mod_name)
}

pub fn ask_for_mod_folder() -> Result<PathBuf, UserInteractionError> {
    println!("Insert the path to the compressed folder of the mod you downloaded\n\
        IT HAS TO BE A COMPRESSED FOLDER (.zip, .7z, .rar)");
    print!("Insert path>> ");
    stdout().flush().map_err(|er| UserInteractionError::StdoutFlush(er))?;

    let mut answer = String::new();
    stdin().read_line(&mut answer).map_err(|er| UserInteractionError::StdinRead(er))?;
    Ok(PathBuf::from(answer.trim()))
}
