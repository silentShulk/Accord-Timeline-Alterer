use std::path::Path;

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
pub struct Args {
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
 
    /// Move the mod's files into 
    /// * hidden folder if it needs to be disabled
    /// * folder for that type if it needs to be enabled
    #[arg(
        long = "toggle",
        short = 't',
        value_name = "NAME",
        help = "Toggle a mod's enabled state by its name"
    )]
    toggle: Option<String>,
 
    /// Update the setting identified by `NAME` to `VALUE` and persist the change
    #[arg(long="settings", short='s', value_names = ["NAME", "VALUE"],
        help="Path to the settings file")]
    setting: Option<Vec<String>>,
 
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
    #[arg(long = "paths", short = 'p', help = "List all of ATA's files")]
    paths: bool,
}
impl Args {
    pub fn get_command(&self) -> Option<Command<'_>> {
        if let Some(p) = &self.install {
            Some(Command::Install(Path::new(&p[0]), p[1].as_str(), self.overwrite))
        } else if let Some(name) = &self.uninstall {
            Some(Command::Uninstall(name))
        } else if self.list_mods {
            Some(Command::ListMods)
        } else if let Some(name) = &self.toggle {
            Some(Command::Toggle(name))
        } else if let Some(p) = &self.setting {
            Some(Command::Setting(&p[0], &p[1]))
        } else if self.list_settings {
            Some(Command::ListSettings)
        } else if self.automata {
            Some(Command::Automata)
        } else if self.paths {
            Some(Command::Paths)
        } else {
            None
        }
    }
}

pub enum Command<'a> {
    Install(&'a Path, &'a str, bool),
    Uninstall(&'a str),
    ListMods,
    Toggle(&'a str),
    Setting(&'a str, &'a str),
    ListSettings,
    Automata,
    Paths,
}