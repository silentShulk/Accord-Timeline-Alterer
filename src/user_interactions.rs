use std::process::Command;

use thiserror::Error;



#[derive(Error, Debug)]
pub enum CommandError {
    #[error("{0} was killed with a system signal")]
    SignalKill(String),
    
    #[error("{0} crashed with exit code {1}")]
    CommandCrash(String, i32)
}



pub fn ask_user_action() -> Result<String, std::io::Error> {
    let fzf_output = Command::new("fzf")
        .arg("--prompt=\"What do you want to do?\"")
        .arg("--height=10%")
        .status();
    
    match fzf_output {
        Ok(exit_status) => {
            if !exit_status.success() {
                if let Some(code) = exit_status.code() {
                    return Err(std::io::Error::other(format!("fzf exited with code {}", code)));
                } else {
                    return Err(std::io::Error::other("fzf was killed by a signal"));
                }
            }
        }
        Err(er) => {}
    }
    
    return Ok(String::new())
}