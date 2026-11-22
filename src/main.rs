//! **main** is the entry point for ATA
//!
//! Parses CLI arguments and dispatches to the correct subcommand.
//! Each subcommand prints its result as JSON to stdout so the
//! caller can consume it programmatically.
//!
//! This includes:
//! * **install**: `--install [PATH] [NAME]` — install a mod from a compressed archive
//! * **uninstall**: `--uninstall [NAME]` — remove an installed mod by name
//! * **list**: `--list-mods` — print all installed mods as a JSON array
//! * **enable**: `--enable [NAME]` — re-activate a disabled mod
//! * **disable**: `--disable [NAME]` — deactivate an enabled mod without removing it
//! * **settings**: `--settings [NAME] [VALUE]` — update a single setting by name
//! * **automata**: `--automata` — launch NieR:Automata via Steam
//! * **files**: `--files` — print application paths as JSON
//!
//! Main function: [`main`]

mod cli;
use cli::{Args, Command};
use ata::data::{mods::Mods, paths::PATHS, settings::Settings};
use ata::{
    features::{
        installation::install_mod,
        misc::launch_automata,
        mod_managing::{toggle_mod, list_mods},
        uninstallation::uninstall_mod,
    },
    utils::json::json
};

use clap::Parser;


 
/// Loads persisted state, dispatches to the requested subcommand, and prints the result as JSON
///
/// Exits with a status code of `1` if loading data/settings fails, or `0` if a command fails execution.
fn main() {
    let args = Args::parse();
    let commands = args.get_command();
    
    let mut mods = Mods::load_data().unwrap_or_else(|er| {
        eprintln!("Problem loading data: {}", er);
        std::process::exit(2);
    });
    let mut settings = Settings::load_settings().unwrap_or_else(|er| {
        eprintln!("Problem loading settings: {}", er);
        std::process::exit(2);
    });

    match commands {
        Some(Command::Install(mod_archive_path, answered_name, overwrite_flag)) => {
            let installed_mod = install_mod(
                mod_archive_path, answered_name, overwrite_flag, &settings, &mut mods
            ).unwrap_or_else(|err| {
                eprintln!("Install Failed: {}", err);
                std::process::exit(1);
            });
            println!("{}", json(&[installed_mod]));
        },
        Some(Command::Uninstall(mod_name)) => {
            let uninstalled_mod = uninstall_mod(
                mod_name, &mut mods
            ).unwrap_or_else(|err| {
                eprintln!("Uninstall Failed: {}", err);
                std::process::exit(1);
            });
            println!("{}", json(&[uninstalled_mod]));
        },
        Some(Command::ListMods) => {
            let listed_mods = list_mods(
                &settings.sorting_order, &mut mods.mods
            );
            println!("{}", json(&[listed_mods]));
        },
        Some(Command::Toggle(mod_name)) => {
            let toggled_mod = toggle_mod(
                mod_name, &mut mods
            ).unwrap_or_else(|err| {
                eprintln!("Toggle Failed: {}", err);
                std::process::exit(1);
            });
            println!("{}", json(&[toggled_mod]));
        },
        Some(Command::Setting(setting, value)) => {
            let updated_settings = settings.update_setting(
                setting, value
            ).unwrap_or_else(|err| {
                eprintln!("Install Failed: {}", err);
                std::process::exit(1);
            });
            println!("{}", json(&[updated_settings]));
        },
        Some(Command::ListSettings) => print!("{}", json(&settings)),
        Some(Command::Automata) => {
            launch_automata().unwrap_or_else(|er| eprintln!("Game failed to launch. {}", er));
            println!("Game starting...");
        },
        Some(Command::Paths) => print!("{}", json(&*PATHS)),
        None => { eprintln!("No commands given!") }
    }
}