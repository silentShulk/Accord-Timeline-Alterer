use std::error::Error;

use std::env::var;

use std::io::{stdin, stdout, Write};

use std::process::{Command, ExitStatus};

use std::path::PathBuf;



// CHECKING GAME PATH LOCATION
pub fn ask_for_correct_gamepath() -> Result<PathBuf, std::io::Error> {
    println!("Insert the correct path to the game's executable");
    print!("Correct path: ");
    stdout().flush()?;

    let mut new_path = String::new();
    stdin().read_line(&mut new_path)?;

    Ok(PathBuf::from(new_path.trim()))
}

// IF MODDING FILES AREN'T PRESENT, WARN THE USER
pub fn missing_files_warning(missing_files: Vec<PathBuf>) -> Result<bool, std::io::Error> {
	for missing_file in missing_files {
		println!("{:?} is missing", missing_file)
	}
    println!("You need to install the file(s) if you want to mod the game");
    
    print!("Start installation of required modding files? [Y/n] ");
    stdout().flush()?;

    let mut answer = String::new();
    stdin().read_line(&mut answer)?;
    let answer = answer.trim();
    
    Ok(answer.is_empty() || answer.eq_ignore_ascii_case("y"))
}

// IF THE USER WANTS TO, INSTALL THE FILES
pub fn run_auto_install_script() -> Result<ExitStatus, Box<dyn Error>> {
    let home_dir = var("HOME").unwrap_or(String::from("/home/2B/"));
    let script_path = PathBuf::from(format!("{}/.local/share/ATA/install-prerequisites.sh", home_dir));

    let status = Command::new("xdg-terminal-exec")
        .arg("-e")
        .arg("bash")
        .arg(&script_path)
        .status();

    match status {
        Ok(exit_status) => Ok(exit_status),
        Err(er) => Err(Box::new(er))
    }
}

pub fn ask_user_action() -> Result<String, std::io::Error> {
    // Asking what the user wants to do
    println!(
        "What do you want to do?\n
            \t1 - Install a mod (you have to provide a zip folder of the mod)
            \t2 - Uninstall a mod (you have to type the name of the mod)
            \t3 - List all mods
            \t0 - Close ATA"
    );
    print!("\nInsert a number: ");
    stdout().flush()?;

    // Getting the user's action's id
    let mut answer = String::new();
    stdin().read_line(&mut answer)?;
    Ok(answer.trim().to_string())
}

pub fn ask_for_mod_folder() -> Result<PathBuf, std::io::Error> {
    println!("To install a mod type the path to the compressed folder of a mod you downloaded\n\
        IT HAS TO BE A COMPRESSED FOLDER (.zip, .7z, .rar)");
    print!("Insert path >> ");
    stdout().flush()?;

    let mut answer = String::new();
    stdin().read_line(&mut answer)?;
    Ok(PathBuf::from(answer.trim()))
}