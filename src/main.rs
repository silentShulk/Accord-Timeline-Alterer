//! **main** is the entry point of the `ATA` command line interface
//!
//! Parses CLI arguments and dispatches to the correct feature of the [`ata`] library.
//! Each command prints its result as JSON to stdout so the caller (usually a GUI)
//! can consume it programmatically. Errors and warnings go to stderr.
//!
//! Commands:
//! * **install**: `--install [PATH] [NAME]` — install a mod from an archive or a folder (`--overwrite` to replace conflicting files)
//! * **uninstall**: `--uninstall [NAME]` — remove an installed mod by name
//! * **list**: `--list-mods` — print all installed mods as a JSON array
//! * **enable**: `--enable [NAME]` — re-activate a disabled mod
//! * **disable**: `--disable [NAME]` — deactivate an enabled mod without removing it
//! * **settings**: `--settings [NAME] [VALUE]` — update a single setting by name
//! * **list settings**: `--list-settings` — print all settings
//! * **automata**: `--automata` — launch NieR:Automata via Steam
//! * **files**: `--files` — print application paths as JSON
//! * **detect game**: `--detect-game` — look for the game in the Steam libraries
//! * **wipe**: `--wipe` — uninstall every mod
//!
//! Exit codes: see [`ExitCode`]

use ata::features::game::{detect_game_path, launch_automata};
use ata::features::installation::{InstallStep, InstallationError, install_mod};
use ata::features::listing::list_mods;
use ata::features::mod_managing::{disable_mod, enable_mod};
use ata::features::uninstallation::uninstall_mod;
use ata::features::wipe::uninstall_all;
use ata::{Mods, PATHS, Settings};

use std::fmt::Display;
use std::path::PathBuf;

use clap::{ArgGroup, Parser};

/// Exit codes of the CLI, so callers can react without parsing error messages
#[repr(i32)]
#[derive(Clone, Copy)]
enum ExitCode {
    /// The data or settings file couldn't be loaded
    LoadFailure = 1,
    // 2 is used by clap for invalid arguments
    /// The command failed
    CommandFailure = 3,
    /// The installation was refused because of conflicts (retry with `--overwrite`)
    InstallConflicts = 4,
}

/// CLI argument definitions for ATA
///
/// Exactly one action flag must be provided per invocation.
/// Clap validates argument counts and generates `--help` output automatically.
#[derive(Parser)]
#[command(
    name = "ATA",
    version,
    about = "Accord's Timeline Alterer, the cross-platform NieR:Automata mod manager",
    long_about = None,
    group(ArgGroup::new("action").required(true).multiple(false).args([
        "install", "uninstall", "list_mods", "enable", "disable", "settings",
        "list_settings", "automata", "files", "detect_game", "wipe",
    ]))
)]
struct Args {
    /// Install a mod from an archive (.zip, .7z, .rar) or a folder at `PATH` and register it under `NAME`
    #[arg(long = "install", short = 'i', num_args = 2, value_names = ["PATH", "NAME"], allow_hyphen_values = true)]
    install: Option<Vec<String>>,

    /// Overwrite files of installed mods conflicting with the one being installed
    #[arg(long = "overwrite", short = 'o', requires = "install")]
    overwrite: bool,

    /// Print every installation step to stderr
    #[arg(long = "verbose", short = 'v', requires = "install")]
    verbose: bool,

    /// Uninstall a mod by its name, restoring the game files it replaced
    #[arg(long = "uninstall", short = 'u', value_name = "NAME", allow_hyphen_values = true)]
    uninstall: Option<String>,

    /// List all installed mods (sorted according to the settings)
    #[arg(long = "list-mods", short = 'm')]
    list_mods: bool,

    /// Enable a mod by its name
    #[arg(long = "enable", short = 'e', value_name = "NAME", allow_hyphen_values = true)]
    enable: Option<String>,

    /// Disable a mod by its name (its files are moved to `.disabled/` folders)
    #[arg(long = "disable", short = 'd', value_name = "NAME", allow_hyphen_values = true)]
    disable: Option<String>,

    /// Change the setting `NAME` to `VALUE` (names are the camelCase keys of settings.json)
    #[arg(long = "settings", short = 's', num_args = 2, value_names = ["NAME", "VALUE"], allow_hyphen_values = true)]
    settings: Option<Vec<String>>,

    /// List all settings and their values
    #[arg(long = "list-settings", short = 'l')]
    list_settings: bool,

    /// Start NieR:Automata through Steam
    #[arg(long = "automata", short = 'a')]
    automata: bool,

    /// List all of ATA's files and folders
    #[arg(long = "files", short = 'f')]
    files: bool,

    /// Look for NieR:Automata in the Steam libraries and print its folder
    #[arg(long = "detect-game")]
    detect_game: bool,

    /// Uninstall every mod (no short flag on purpose)
    #[arg(long = "wipe")]
    wipe: bool,
}

/// Dispatches to the requested command, loading only the state it needs
fn main() {
    let args = Args::parse();

    if let Some(params) = args.install {
        let (settings, mut data) = (load_settings(), load_data());
        let (path, name) = (PathBuf::from(&params[0]), &params[1]);
        let verbose = args.verbose;
        let mut report = |step: InstallStep, message: &str| match step {
            InstallStep::Warning => eprintln!("warning: {message}"),
            _ if verbose => eprintln!("[{step:?}] {message}"),
            _ => {}
        };

        match install_mod(&path, name, args.overwrite, &settings, &mut data, &mut report) {
            Ok(installed) => print_json(&[installed]),
            Err(err @ InstallationError::FileConflict(_)) => fail("Install failed", err, ExitCode::InstallConflicts),
            Err(err) => fail("Install failed", err, ExitCode::CommandFailure),
        }
    } else if let Some(name) = args.uninstall {
        let (settings, mut data) = (load_settings(), load_data());
        let uninstalled = uninstall_mod(&mut data, &name, &settings.game_path).unwrap_or_else(|er| command_failed("Uninstall failed", er));
        print_json(&[uninstalled]);
    } else if args.list_mods {
        let (settings, data) = (load_settings(), load_data());
        print_json(&list_mods(settings.sorting_order, &data.mods));
    } else if let Some(name) = args.enable {
        let (settings, mut data) = (load_settings(), load_data());
        let enabled = enable_mod(&mut data, &name, &settings.game_path).unwrap_or_else(|er| command_failed("Enable failed", er));
        print_json(&[enabled]);
    } else if let Some(name) = args.disable {
        let (settings, mut data) = (load_settings(), load_data());
        let disabled = disable_mod(&mut data, &name, &settings.game_path).unwrap_or_else(|er| command_failed("Disable failed", er));
        print_json(&[disabled]);
    } else if args.list_settings {
        print_json(&load_settings());
    } else if let Some(params) = args.settings {
        let mut settings = load_settings();
        let changed = settings
            .update_setting(&params[0], &params[1])
            .unwrap_or_else(|er| command_failed("Settings change failed", er));
        print_json(&[changed]);
    } else if args.automata {
        launch_automata().unwrap_or_else(|er| command_failed("Game failed to launch", er));
        print!("Game starting...");
    } else if args.files {
        print_json(&*PATHS);
    } else if args.detect_game {
        match detect_game_path() {
            Some(path) => print_json(&path),
            None => command_failed("Detection failed", "NieR:Automata wasn't found in any Steam library"),
        }
    } else if args.wipe {
        let (settings, mut data) = (load_settings(), load_data());
        let report = uninstall_all(&mut data, &settings.game_path);
        print_json(&report);
        if !report.failed.is_empty() {
            std::process::exit(ExitCode::CommandFailure as i32);
        }
    }
}

/// Loads the installed mods, exiting with [`ExitCode::LoadFailure`] on failure
fn load_data() -> Mods {
    Mods::load_data().unwrap_or_else(|er| fail("Problem loading data", er, ExitCode::LoadFailure))
}

/// Loads the settings, exiting with [`ExitCode::LoadFailure`] on failure
fn load_settings() -> Settings {
    Settings::load_settings().unwrap_or_else(|er| {
        fail(
            "Problem loading settings (the ATA Launcher can create/repair them)",
            er,
            ExitCode::LoadFailure,
        )
    })
}

/// Reports a failed command and exits with [`ExitCode::CommandFailure`]
fn command_failed(context: &str, error: impl Display) -> ! {
    fail(context, error, ExitCode::CommandFailure)
}

/// Prints `context: error` to stderr and exits with `code`
fn fail(context: &str, error: impl Display, code: ExitCode) -> ! {
    eprintln!("{context}: {error}");
    std::process::exit(code as i32);
}

/// Serializes `value` to compact JSON on stdout
///
/// Panics if `value` cannot be serialized — this should never happen for
/// the types used in this codebase.
fn print_json<T: serde::Serialize + ?Sized>(value: &T) {
    print!("{}", serde_json::to_string(value).expect("ATA's types always serialize to JSON"));
}
