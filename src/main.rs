use clap::Parser;

mod data_config;
use data_config::{Config};

mod user_interactions;
use user_interactions::{
    ask_user_action
};

mod features;
use features::{install_mod, uninstall_mod, list_mods};

mod installation_functions;
use installation_functions::ask_for_mod_folder;

mod uninstallation_functions;
use uninstallation_functions::ask_for_mod_name;



fn main() {
    println!("\nWELCOME TO ACCORD'S TIMELINE ALTERER\n(AUTOMATA'S MOD MANAGER FOR LINUX)\n\n");



    /* ----------------------- */
    /*   LOADING CONFIG DATA   */
    /* ----------------------- */

    println!("Loading data file (~/.config/ATA/data.json)");
    
    let mut current_config = Config::load_config()
    .unwrap_or_else(|err| {
        eprintln!("There was a problem accessing the data file (~/.config/ATA/data.json). {}\nConsider checking if the file is there and if it isn't corrupted.
                ATA will now close...", err);
        std::process::exit(1);
    });
    
    println!("Config file (~/.config/ATA/data.json) loaded!\n");
    


    /* -------------------------------- */
    /*   STARTING ONE OF THE FEATURES   */
    /* -------------------------------- */
    
    clearscreen::clear().unwrap_or_else(|er| {
   		eprintln!("There was a problem clearing the console screen. {}
     			ATA will now close...", er)
    });

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

        	let installed_mod = install_mod(&current_config.game_path, &answered_path).unwrap_or_else(|er| {
             	eprintln!("There was a problem installing the mod. {}
                        ATA will now close...", er);
               	std::process::exit(1);
            });
         
        	current_config.save_new_mod(installed_mod).unwrap_or_else(|er| {
       			eprintln!("Therer was a problem adding the newly installed mod to the data file. {}
          				ATA will now close...", er);
         	});
         
            println!("MOD INSTALLED");   
        }
        // UNINSTALL A MOD
        else if action_id == "2" {
            let mod_to_uninstall = ask_for_mod_name().unwrap_or_else(|er| {
                eprintln!("There was a problem using the console for asking for the compressed mod folder. {}
                        ATA will now close...", er);
                std::process::exit(1);
            });
            
        	uninstall_mod(&current_config.game_path, &current_config.mods, mod_to_uninstall).unwrap_or_else(|er| {
            	eprintln!("There was a problem uninstalling the mod. {}
                        ATA will now close...", er);
              	std::process::exit(1);
            });
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
