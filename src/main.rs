//! **main** is the module that contains the main functions
//! Which branches into a separate function based on the args received
//! 
//! This includes:
//! * **installing a mod**: --install [PATH TO MOD ARCHIVE] [NAME *FOR* MOD]
//! * **uninstalling a mod**: --uninstall [NAME OF MOD]
//! * **listing installed mods**: --list
//! * **enabling a mod**: --enable [NAME OF MOD]
//! * **disabling a mod**: --disable [NAME OF MOD]
//! 
//! Main function: [`main`]



mod data;
mod installation;
mod uninstallation;
mod mod_managing;
mod settings;

use data::Data;
use installation::install_mod;
use uninstallation::uninstall_mod;
use mod_managing::{enable_mod, disable_mod};
use settings::Settings;

use std::path::PathBuf;

use clap::Parser;



/// 
#[derive(Parser)]
#[command(name = "ATA", version = "0.01", about = "Accord's Timeline Alterer, the NieR Automata mod manager for Linux")]
struct Args {
    #[arg(long = "install", short='i', num_args = 2, value_names = ["PATH", "NAME"],
        help="Install a mod from a given path with a specified name")]
    install: Option<Vec<String>>,

    #[arg(long="uninstall", short='u', value_name = "NAME",
        help="Uninstall a mod by its name")]
    uninstall: Option<String>,

    #[arg(long="list", short='l',
        help="List all installed mods")]
    list: bool,

    #[arg(long="enable", short='e', value_name = "NAME",
        help="Enable a mod by its name")]
    enable: Option<String>,

    #[arg(long="disable", short='d', value_name = "NAME",
        help="Disable a mod by its name")]
    disable: Option<String>,

    #[arg(long="settings", short='s', value_names = ["NAME", "VALUE"],
        help="Path to the settings file")]
    settings: Option<Vec<String>>,
}



fn main() {
    let args = Args::parse();

    let mut data = Data::load_data().unwrap_or_else(|err| {
        eprintln!("Problem loading config: {}", err);
        std::process::exit(1);
    });
    let mut settings = Settings::load_settings().unwrap_or_else(|err| {
        eprintln!("Problem loading settings: {}", err);
        std::process::exit(1);
    });

    if let Some(params) = args.install {
        let installed_mod = install_mod(&PathBuf::from(&params[0]), &mut data, &settings.game_path, params[1].clone())
            .unwrap_or_else(|er| { eprintln!("Install failed: {}", er); std::process::exit(1); });
        println!("{}", json(&[installed_mod]));
    }
    else if let Some(name) = args.uninstall {
        let uninstalled_mod = uninstall_mod(&mut data, name)
            .unwrap_or_else(|er| { eprintln!("Uninstall failed: {}", er); std::process::exit(1); });
        println!("{}", json(&[uninstalled_mod]));
    }
    else if args.list {
        println!("{}", json(&data.mods));
    }
    else if let Some(name) = args.enable {
        let enabled_mod = enable_mod(&mut data, name)
            .unwrap_or_else(|er| { eprintln!("Enable failed: {}", er); std::process::exit(1); });
        println!("{}", json(&[enabled_mod]));
    }
    else if let Some(name) = args.disable {
        let disabled_mod = disable_mod(&mut data, name)
            .unwrap_or_else(|er| { eprintln!("Disable failed: {}", er); std::process::exit(1); });
        println!("{}", json(&[disabled_mod]));
    }
    else if let Some(params) = args.settings {
        let changed_setting = settings.update_setting(params[0].clone(), params[1].clone())
            .unwrap_or_else(|er| { eprintln!("Settings Change failed: {}", er); std::process::exit(1); });
        println!("{}", json(&[changed_setting]));
    }
    else {
        eprintln!("No command given");
        std::process::exit(1);
    }
}



fn json<T>(mod_returned: &T) -> String 
where
    T: serde::Serialize,
{
    let json = serde_json::to_string(mod_returned).unwrap();
    
    json
}