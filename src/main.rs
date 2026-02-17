use std::error::Error;

use clap::Parser;

use clearscreen::clear;

mod data_saving;
use data_saving::{Config, Mod};

mod starting_checks;
use starting_checks::{
    check_path, check_for_required_modding_files
};

mod user_interactions;
use user_interactions::{
    ask_for_correct_gamepath, missing_files_warning, run_auto_install_script, ask_user_action, ask_for_mod_folder
};

mod features;
use features::{install_mod, uninstall_mod, list_mods};

mod installation_utilities_and_methods;



fn main() {
    println!("\nWELCOME TO ACCORD'S TIMELINE ALTERER\n(AUTOMATA'S MOD MANAGER FOR LINUX)\n\n");



    /* ----------------------- */
    /*   LOADING CONFIG DATA   */
    /* ----------------------- */

    // LOAD DATA IF PRESENT
    println!("Loading data file (~/.config/ATA/data.json)");
    
    let mut current_config = Config::load_config()
    .unwrap_or_else(|err| {
        eprintln!("There was a problem accessing the data file (~/.config/ATA/data.json). {}\nConsider checking if the file is there and if it isn't corrupted.
                ATA will now close...", err);
        std::process::exit(1);
    });
    
    println!("Config file (~/.config/ATA/data.json) loaded!\n");
    


    /* ------------------- */
    /*   STARTING CHECKS   */
    /* ------------------- */
    
    // CHECKING GAME PATH LOCATION
    println!("Checking if the currently saved gamepath is the correct one (contains the game's files)");
    
    let mut path_is_valid = false;
    while !path_is_valid {
        // Accessing the given path and checking if it actually contains the game's files
        let is_gamepath = check_path(&current_config.game_path).unwrap_or_else(|er| {
            eprintln!("There has been a problem checking the given game path. {}
                    ATA will now close...", er);
            std::process::exit(1);
        });
        
        // If the path is incorrect, ask the user for another one
        if !is_gamepath {
            println!("Game installation not found at {:?} (it doesn't contain NieRAutomata.exe)", current_config.game_path);
            
            current_config.game_path = ask_for_correct_gamepath()
                .unwrap_or_else(|er| {
                    eprintln!("There has been a problem trying to change the game path. {}
                            ATA will now close...", er);
                    std::process::exit(1);
                });
        }
        
        path_is_valid = is_gamepath;
    }
        
    println!("Game installation found at {:?}\n", current_config.game_path);

    
    
    // CHECKING IF THE REQUIRED MODDING FILES ARE INSTALLED
    println!("Checking if the required modding files are installed");
    
    let missing_required_modding_files = check_for_required_modding_files(&current_config.game_path);
    if missing_required_modding_files.len() > 0 {
        let user_answer = missing_files_warning(missing_required_modding_files)
            .unwrap_or_else(|er| {
                eprintln!("There has been a problem using the console to warn you about the missing required modding files. {}
                        ATA will now close...", er);
                std::process::exit(1);
            });
        
        if user_answer {
            run_auto_install_script().unwrap_or_else(|er| {
                eprintln!("There has been a problem running the installation script for the required modding files. {}
                        ATA will now close...", er);
                std::process::exit(1);
            });
            
            println!("Required modding files installed successfully!");
        }
        else {
            eprint!("Cannot proceed further without the required modding files.
                    ATA will now close...");
            std::process::exit(1);
}
    } else {
        println!("Required modding files already installed")
    }
    
    clearscreen::clear().unwrap_or_else(|er| {
        println!("There has been a problem trying to clear the terminal screeen. {}
                ATA will now close...", er);
        std::process::exit(1);
    });
    
    
    
    /* -------------------------------- */
    /*   STARTING ONE OF THE FEATURES   */
    /* -------------------------------- */

    let mut action_id = String::from("");
    while action_id != "0" {
        action_id = ask_user_action().unwrap_or_else(|er| {
            eprintln!("There has been a problem using the console to ask you what you want to do. {}
                    ATA will now close...", er);
            std::process::exit(1);
        });

        // INSTALL A MOD
        if action_id == "1" {
            let answered_path = ask_for_mod_folder().unwrap_or_else(|er| {
                eprintln!("There was a problem using the console for asking for the compressed mod folder. {}
                        ATA will now close...", er);
                std::process::exit(1);
            });

        	let installed_mod = install_mod(&current_config.game_path, answered_path).unwrap_or_else(|er| {
             	eprintln!("There was a problem installing the mod. {}", er);
               	std::process::exit(1);
            });
            println!("MOD INSTALLED");
                    
            save_mod_data(installed_mod).unwrap_or_else(|er| {
                println!("There was an error saving the data of the installed mod to the data file (~/.config/ATA/data.json). {}", er);
                std::process::exit(1);
            });
        }
        // UNINSTALL A MOD
        else if action_id == "2" {
        	uninstall_mod(&current_config.game_path);
        } 
        // PRINT THE LIST OF INSTALLED MODS
        else if action_id == "3" {
            list_mods(&current_config.mods);
        }
        // EXIT THE PROGRAM
        else if action_id == "0" {
            println!("Happy Automata (ATA will now close...)");
            std::process::exit(1);
        }
        else {
            println!("\"{}\" is not a valid action id (input either 1, 2, 3 or 0)", action_id);
        }
    }
}






/* ----- */
/*   SHOULD MOVE TO USER INTERACTION FILW WITH MISSING FILES WQARING   */
/* ----- */



fn save_mod_data(mod_data: Mod) -> Result<(), Box<dyn Error>> {
	Ok(())
}






/* ---------------------------- */
/*   FLAGS FOR QUICK FEATURES   */
/* ---------------------------- */

#[derive(Parser)]
#[command(
    name = "NAMHL",
    version = "0.01",
    about = "The Nier Automata Mod Helper for Linux"
)]
struct Args {
    folder_path: String,
    mod_name: String,
    // Will add arguments here
}
