use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;

use inquire::autocompletion::{Autocomplete, Replacement};
use inquire::{InquireError, Text};
use inquire::prompt_text;
use std::io::Write;

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

    #[error("{0}")]
    InquirePrompt(#[from] InquireError),
}



#[derive(Clone, Default)]
struct ArchivePathAutocomplete;

impl Autocomplete for ArchivePathAutocomplete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let path = std::path::Path::new(input);
        
        let (dir, typed_prefix): (&std::path::Path, &str) = if path.is_dir() {
            (path, "")
        } else {
            let parent = path.parent().unwrap_or(std::path::Path::new("."));
            let prefix = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            (parent, prefix)
        };

        let mut suggestions = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                let is_relevant = entry_path.is_dir()
                    || entry_path.extension().and_then(|e| e.to_str()).map_or(false, |ext| {
                        matches!(ext, "zip" | "7z" | "rar")
                    });

                if !is_relevant {
                    continue;
                }

                let Some(path_str) = entry_path.to_str() else {
                    continue;
                };

                let entry_name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if entry_name.starts_with(typed_prefix) {
                    if entry_path.is_dir() {
                        suggestions.push(format!("{}/", path_str));
                    } else {
                        suggestions.push(path_str.to_owned());
                    }
                }
            }
        }

        suggestions.sort();
        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted: Option<String>,
    ) -> Result<Replacement, inquire::CustomUserError> {
        Ok(highlighted
            .map(Replacement::Some)
            .unwrap_or(Replacement::None))
    }
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

    let options =
        "Install a mod\nUninstall a mod\nList mods\nEnable a mod\nDisable a mod\nClose ATA :(";

    let mut fzf_stdin = fzf_child
        .stdin
        .take()
        .ok_or(UserInteractionError::MissingPipeHandle("fzf".to_string()))?;
    fzf_stdin
        .write_all(options.as_bytes())
        .map_err(|er| UserInteractionError::BufferWriting(er, options.to_string()))?;
    drop(fzf_stdin);

    let fzf_output = fzf_child
        .wait_with_output()
        .map_err(|er| UserInteractionError::ProcessOutputReading("fzf".to_string(), er))?;

    if !fzf_output.status.success() {
        return if let Some(code) = fzf_output.status.code() {
            Err(UserInteractionError::CommandCrash("fzf".to_string(), code))
        } else {
            Err(UserInteractionError::SignalKill("fzf".to_string()))
        };
    }

    let user_selection = String::from_utf8(fzf_output.stdout)?;
    Ok(user_selection.trim().to_string())
}

pub fn ask_user_name_for_mod() -> Result<String, UserInteractionError> {
    let name = prompt_text("Select an identifier for this mod")?;
    Ok(name.trim().to_string())
}

pub fn ask_for_mod_name() -> Result<String, UserInteractionError> {
    let name = prompt_text("Insert the name of a mod")?;
    Ok(name.trim().to_string())
}

pub fn ask_for_mod_folder() -> Result<PathBuf, UserInteractionError> {
    let path_str = Text::new("Path to the compressed mod folder (.zip / .7z / .rar):")
        .with_autocomplete(ArchivePathAutocomplete)
        .prompt()?;

    Ok(PathBuf::from(path_str.trim()))
}
