mod data_config;
use data_config::{Config};

mod installation;
mod uninstallation;
mod installed_mod_managing;

use installation::install_mod;
use uninstallation::uninstall_mod;
use installed_mod_managing::{list_mods, enable_mod, disable_mod};

mod user_interactions;
use user_interactions::{
    ask_user_action,
    ask_for_mod_name,
    ask_for_mod_folder
};

use clap::Parser;



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
    while action_id != "Close ATA :(" {
        action_id = ask_user_action().unwrap_or_else(|er| {
            eprintln!("There has been a problem using the console to ask you what you want to do. {}
                    ATA will now close...", er);
            std::process::exit(1);
        });
        
        clearscreen::clear().unwrap_or_else(|er| {
       		eprintln!("There was a problem clearing the console screen. {}
         			ATA will now close...", er)
        });

        // INSTALL A MOD
        if action_id == "Install a mod" {
            let answered_path = ask_for_mod_folder().unwrap_or_else(|er| {
                eprintln!("There was a problem using the console for asking for the compressed mod folder. {}
                        ATA will now close...", er);
                std::process::exit(1);
            });

        	let installed_mod = install_mod(&mut current_config, &answered_path).unwrap_or_else(|er| {
             	eprintln!("There was a problem installing the mod. {}
                        ATA will now close...", er);
               	std::process::exit(1);
            });

            println!("\nMOD INSTALLED");
            println!("{}", installed_mod);
        }
        // UNINSTALL A MOD
        else if action_id == "Uninstall a mod" {
            let mod_to_uninstall = ask_for_mod_name().unwrap_or_else(|er| {
                eprintln!("There was a problem using the console for asking for the compressed mod folder. {}
                        ATA will now close...", er);
                std::process::exit(1);
            });

        	let uninstalled_mod_index = uninstall_mod(&mut current_config, mod_to_uninstall).unwrap_or_else(|er| {
            	eprintln!("There was a problem uninstalling the mod. {}
                        ATA will now close...", er);
              	std::process::exit(1);
            });

            current_config.remove_mod(uninstalled_mod_index).unwrap_or_else(|er| {
                eprintln!("There was a problem removing the newly installed mod to the data file. {}
                        ATA will now close...", er);
                std::process::exit(1);
            })
        }
        // PRINT THE LIST OF INSTALLED MODS
        else if action_id == "List mods" {
            list_mods(&current_config.mods);
        }
        else if action_id == "Enable a mod" {
            let name = ask_for_mod_name().unwrap_or_else(|er| {
                eprintln!("There was a problem using the console for asking for the compressed mod folder. {}
                        ATA will now close...", er);
                std::process::exit(1);
            });

            let enabled_mod = enable_mod(&mut current_config, name).unwrap_or_else(|er| {
                eprintln!("There was a problem enabling the mod. {}
                        ATA will now close...", er);
                std::process::exit(1);
            });
            
            println!("\nENABLED: {}", enabled_mod);
        }
        else if action_id == "Disable a mod" {
        	let name = ask_for_mod_name().unwrap_or_else(|er| {
            	eprintln!("There was a problem using the console for asking for the compressed mod folder. {}
                    	ATA will now close...", er);
             	std::process::exit(1);
         	});

            let disabled_mod = disable_mod(&mut current_config, name).unwrap_or_else(|er| {
                eprintln!("There was a problem disabling the mod. {}
                        ATA will now close...", er);
                std::process::exit(1);
            });
            
            println!("\nDISABLED: {}", disabled_mod);
        }
        else if action_id != "Close ATA :(" {
            clearscreen::clear().unwrap_or_else(|er| {
           		eprintln!("There was a problem clearing the console screen. {}
             			ATA will now close...", er)
            });

            println!("\"{}\" is not a valid action id (Select one of the options displayed in the menu)", action_id);
        }
    }

    println!("Thank you for using ATA.\n\t\tHappy Automata :)")
}






/* ---------------------------- */
/*   FLAGS FOR QUICK FEATURES   */
/* ---------------------------- */

#[derive(Parser)]
#[command(
    name = "ATA",
    version = "0.01",
    about = "Accord's Timeline Alterer, the NieR Automata mod manager for Linux"
)]
struct Args {
    folder_path: String,
    mod_name: String,
    // Will add arguments here
}
