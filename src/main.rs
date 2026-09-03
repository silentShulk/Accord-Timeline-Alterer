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

mod saved_data;
mod features;
mod utils;
use saved_data::{data, settings, paths};
use features::{installation, uninstallation, mod_managing, misc};

use paths::PATHS;
use data::Data;
use settings::Settings;
use uninstallation::uninstall_mod;
use mod_managing::{disable_mod, enable_mod, list_mods};
use installation::install_mod;
use misc::{/*update_discord_rich_presence, Action,*/ launch_automata};

use std::path::PathBuf;

use clap::Parser;


 
/// CLI argument definitions for ATA
///
/// Exactly one primary action flag should be provided per invocation.
/// Clap validates argument counts and generates `--help` output automatically.
#[derive(Parser)]
#[command(
    name = "ATA",
    version = "0.01",
    about = "Accord's Timeline Alterer, the cross-platform NieR Automata mod manager"
)]
struct Args {
    /// Install a mod from a compressed archive at `PATH` and register it under `NAME`
    #[arg(long = "install", short='i', num_args = 2, value_names = ["PATH", "NAME"],
        help="Install a mod from a given path with a specified name")]
    install: Option<Vec<String>>,
 
    /// Forces any older conflicting mod file to be overwritten during installation
    #[arg(
        long = "overwrite",
        short = 'o',
        help = "Forces any older conflicting file to be overwritten",
        requires = "install"
    )]
    overwrite: bool,
 
    /// Uninstall the mod registered under `NAME`, removing its files from the game directory
    #[arg(
        long = "uninstall",
        short = 'u',
        value_name = "NAME",
        help = "Uninstall a mod by its name"
    )]
    uninstall: Option<String>,
 
    /// Print all installed mods as a JSON array and exit
    #[arg(long = "list-mods", short = 'm', help = "List all installed mods")]
    list_mods: bool,
 
    /// Move the mod's files back into the game directory so the game loads them
    #[arg(
        long = "enable",
        short = 'e',
        value_name = "NAME",
        help = "Enable a mod by its name"
    )]
    enable: Option<String>,
 
    /// Move the mod's files to a `.disabled/` subfolder so the game ignores them
    #[arg(
        long = "disable",
        short = 'd',
        value_name = "NAME",
        help = "Disable a mod by its name"
    )]
    disable: Option<String>,
 
    /// Update the setting identified by `NAME` to `VALUE` and persist the change
    #[arg(long="settings", short='s', value_names = ["NAME", "VALUE"],
        help="Path to the settings file")]
    settings: Option<Vec<String>>,
 
    /// Print all settings and their current values as JSON
    #[arg(
        long = "list-settings",
        short = 'l',
        help = "List all settings and their values"
    )]
    list_settings: bool,
 
    /// Start NieR:Automata via Steam
    #[arg(long = "automata", short = 'a', help = "Start NieR:Automata")]
    automata: bool,
 
    /// List all internal application file paths used by ATA
    #[arg(long = "files", short = 'f', help = "List all of ATA's files")]
    files: bool,
}
 
/// Loads persisted state, dispatches to the requested subcommand, and prints the result as JSON
///
/// Exits with a status code of `1` if loading data/settings fails, or `0` if a command fails execution.
fn main() {
    let args = Args::parse();
    // let mut action = Action::JustOpened;
 
    let mut data = Data::load_data().unwrap_or_else(|er| {
        eprintln!("Problem loading data: {}", er);
        std::process::exit(1);
    });
    let mut settings = Settings::load_settings().unwrap_or_else(|er| {
        eprintln!("Problem loading settings: {}", er);
        std::process::exit(1);
    });
 
    // update_discord_rich_presence(&settings.discord_rich_presence, action).unwrap_or_else(|er| {
    //     eprintln!("Problem using DRP: {}", er);
    // });
 
    if let Some(params) = args.install {
        let overwrite = args.overwrite;
        let installed_mod = install_mod(
            &PathBuf::from(&params[0]),
            params[1].clone(),
            overwrite,
            &settings,
            &mut data,
        )
        .unwrap_or_else(|er| {
            eprintln!("Install failed: {}", er);
            std::process::exit(0);
        });
        print!("{}", json(&[installed_mod]));
        // action = Action::Installing;
    } else if let Some(name) = args.uninstall {
        let uninstalled_mod = uninstall_mod(&mut data, name).unwrap_or_else(|er| {
            eprintln!("Uninstall failed: {}", er);
            std::process::exit(0);
        });
        print!("{}", json(&[uninstalled_mod]));
        // action = Action::Uninstalling
    } else if args.list_mods {
        let sorted_mods = list_mods(&settings.sorting_order, &data.mods);
        print!("{}", json(&sorted_mods));
        // action = Action::ListingMods;
    } else if let Some(name) = args.enable {
        let enabled_mod = enable_mod(&mut data, name).unwrap_or_else(|er| {
            eprintln!("Enable failed: {}", er);
            std::process::exit(0);
        });
        print!("{}", json(&[enabled_mod]));
        // action = Action::Enabling;
    } else if let Some(name) = args.disable {
        let disabled_mod = disable_mod(&mut data, name).unwrap_or_else(|er| {
            eprintln!("Disable failed: {}", er);
            std::process::exit(0);
        });
        print!("{}", json(&[disabled_mod]));
        // action = Action::Disabling;
    } else if args.list_settings {
        print!("{}", json(&settings))
    } else if let Some(params) = args.settings {
        let changed_setting = settings
            .update_setting(params[0].clone(), params[1].clone())
            .unwrap_or_else(|er| {
                eprintln!("Settings Change failed: {}", er);
                std::process::exit(0);
            });
        print!("{}", json(&[changed_setting]));
        // action = Action::ChangingSettings;
    } else if args.automata {
        launch_automata().unwrap_or_else(|er| eprintln!("Game failed to launch. {}", er));
        print!("Game starting...");
        // action = Action::Playing;
    } else if args.files {
        print!("{}", json(&*PATHS))
    } else {
        eprintln!("No command given");
    }
 
    // update_discord_rich_presence(&settings.discord_rich_presence, action).unwrap_or_else(|er| {
    //     eprintln!("Problem using DRP: {}", er);
    // });
}
 
/// Serializes `value` to a compact JSON string and returns it
///
/// Panics if `value` cannot be serialized — this should never happen for
/// the types used in this codebase.
///
/// # Arguments
/// * `value` - Any value that implements [`serde::Serialize`]
///
/// # Returns
/// * A [`String`] containing the compact JSON representation of `value`
fn json<T>(value: &T) -> String
where
    T: serde::Serialize,
{
    serde_json::to_string(value).unwrap()
}