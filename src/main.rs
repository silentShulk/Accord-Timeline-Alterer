//! **main** is the entry point for ATA
//!
//! Parses CLI arguments and dispatches to the correct subcommand.
//! Each subcommand prints its result as JSON to stdout so the Tauri
//! frontend (or any other caller) can consume it programmatically.
//!
//! This includes:
//! * **install**: `--install [PATH] [NAME]` — install a mod from a compressed archive
//! * **uninstall**: `--uninstall [NAME]` — remove an installed mod by name
//! * **list**: `--list` — print all installed mods as a JSON array
//! * **enable**: `--enable [NAME]` — re-activate a disabled mod
//! * **disable**: `--disable [NAME]` — deactivate an enabled mod without removing it
//! * **settings**: `--settings [KEY] [VALUE]` — update a single setting by name
//!
//! Main function: [`main`]



mod data;
mod installation;
mod uninstallation;
mod mod_managing;
mod settings;
mod paths;
mod misc;

use data::Data;
use installation::install_mod;
use uninstallation::uninstall_mod;
use mod_managing::{list_mods, enable_mod, disable_mod};
use settings::Settings;
use misc::{update_discord_rich_presence, Action};

use std::path::PathBuf;

use clap::Parser;



/// CLI argument definitions for ATA
///
/// Exactly one of the flags must be provided per invocation.
/// Clap validates argument counts and generates `--help` output automatically.
#[derive(Parser)]
#[command(name = "ATA", version = "0.01", about = "Accord's Timeline Alterer, the NieR Automata mod manager for Linux")]
struct Args {
    /// Install a mod from a compressed archive at `PATH` and register it under `NAME`
    #[arg(long = "install", short='i', num_args = 2, value_names = ["PATH", "NAME"],
        help="Install a mod from a given path with a specified name")]
    install: Option<Vec<String>>,

    /// Uninstall the mod registered under `NAME`, removing its files from the game directory
    #[arg(long="uninstall", short='u', value_name = "NAME",
        help="Uninstall a mod by its name")]
    uninstall: Option<String>,

    /// Print all installed mods as a JSON array and exit
    #[arg(long="mods", short='m',
        help="List all installed mods")]
    list_mods: bool,

    /// Move the mod's files back into the game directory so the game loads them
    #[arg(long="enable", short='e', value_name = "NAME",
        help="Enable a mod by its name")]
    enable: Option<String>,

    /// Move the mod's files to a `.disabled/` subfolder so the game ignores them
    #[arg(long="disable", short='d', value_name = "NAME",
        help="Disable a mod by its name")]
    disable: Option<String>,

    /// Update the setting identified by `NAME` to `VALUE` and persist the change
    #[arg(long="settings", short='s', value_names = ["NAME", "VALUE"],
        help="Path to the settings file")]
    settings: Option<Vec<String>>,

    #[arg(long="list-settings", short='l',
        help="List all settings and their values")]
    list_settings: bool
}



/// Loads persisted state, dispatches to the requested subcommand, and prints the result as JSON
///
/// Exits with a non-zero status code if loading state fails, if an unrecognised
/// subcommand combination is given, or if the subcommand itself returns an error.
fn main() {
    let args = Args::parse();

    let mut data = Data::load_data().unwrap_or_else(|er| {
        eprintln!("Problem loading config: {}", er);
        std::process::exit(1);
    });
    let mut settings = Settings::load_settings().unwrap_or_else(|er| {
        eprintln!("Problem loading settings: {}", er);
        std::process::exit(1);
    });

    update_discord_rich_presence(&settings.discord_rich_presence, Action::Installing).unwrap_or_else(|er| {
        eprintln!("Problem using DRP: {}", er);
        std::process::exit(1);
    });   

    if let Some(params) = args.install {
        let installed_mod = install_mod(&PathBuf::from(&params[0]), params[1].clone(), &settings, &mut data,)
            .unwrap_or_else(|er| { eprintln!("Install failed: {}", er); std::process::exit(1); });
        println!("{}", json(&[installed_mod]));
    }
    else if let Some(name) = args.uninstall {
        let uninstalled_mod = uninstall_mod(&mut data, name)
            .unwrap_or_else(|er| { eprintln!("Uninstall failed: {}", er); std::process::exit(1); });
        println!("{}", json(&[uninstalled_mod]));
    }
    else if args.list_mods {
        let sorted_mods = list_mods(&settings.sorting_order, &data.mods);
        println!("{}", json(&sorted_mods));
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
    else if args.list_settings {
        println!("{}", json(&settings))
    }
    else {
        eprintln!("No command given");
        std::process::exit(1);
    }
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
