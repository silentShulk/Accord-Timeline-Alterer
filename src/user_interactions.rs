use std::error::Error;

use std::env::var;

use std::io::{stdin, stdout, Write};

use std::process::{Command, ExitStatus};

use std::path::PathBuf;

use crate::installation_utilities_and_methods::InstallationError;



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

pub fn ask_mod_name() -> Result<String, InstallationError> {
	println!("Insert name of the mod that you are installing (choose anything you want, will be used as identifier)");
	print!("Name: ");
	stdout().flush()?;

	let mut answer = String::new();
	stdin().read_line(&mut answer)?;
	Ok(answer)
}