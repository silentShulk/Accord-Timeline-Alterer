//! **game** is a module that contains everything related to the game installation itself
//!
//! This includes:
//! * **launching**: Starting NieR:Automata through Steam
//! * **validating**: Checking that a folder really contains the game
//! * **detecting**: Finding the game inside the user's Steam libraries
//!
//! Main functions: [`launch_automata`], [`validate_game_path`], [`detect_game_path`]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use thiserror::Error;

/// Steam application ID of NieR:Automata
pub const STEAM_APP_ID: &str = "524220";
/// Name of the game executable, found in the game's root folder
pub const GAME_EXECUTABLE: &str = "NieRAutomata.exe";
/// Name of the game folder inside `steamapps/common`
const GAME_FOLDER_NAME: &str = "NieRAutomata";

/// Files whose presence in the game folder means ReShade is installed
/// (ReShade 4.x is shipped as one of these DLLs, next to its `ReShade.ini`)
pub const RESHADE_BINARIES: &[&str] = &["dxgi.dll", "d3d11.dll", "d3d10.dll", "d3d9.dll", "opengl32.dll"];

/// Launches NieR:Automata via Steam protocol standard (`steam://run/524220`)
///
/// Uses `xdg-open` on Linux (falling back to the `steam` command) and `cmd /C start` on Windows.
/// The launcher's standard streams are detached: otherwise a GUI reading ATA's output would
/// wait for Steam itself to exit, since it would inherit them.
///
/// # Errors
/// * Returns [`std::io::Error`] if the OS process launcher cannot be executed
pub fn launch_automata() -> Result<(), std::io::Error> {
    let steam_url = format!("steam://run/{STEAM_APP_ID}");

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        detached(Command::new("cmd").args(["/C", "start", "", &steam_url]))
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?;
    }

    #[cfg(not(windows))]
    {
        if detached(Command::new("xdg-open").arg(&steam_url)).spawn().is_err() {
            detached(Command::new("steam").arg(&steam_url)).spawn()?;
        }
    }

    Ok(())
}

/// Detaches the standard streams of a command
fn detached(command: &mut Command) -> &mut Command {
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
}

/// Checks that `game_path` points to a NieR:Automata installation
///
/// # Errors
/// * [`GamePathError::Unset`] if the path is empty
/// * [`GamePathError::NotFound`] if the folder doesn't exist
/// * [`GamePathError::NotTheGame`] if the folder doesn't contain the game executable
pub fn validate_game_path(game_path: &Path) -> Result<(), GamePathError> {
    if game_path.as_os_str().is_empty() {
        Err(GamePathError::Unset)
    } else if !game_path.is_dir() {
        Err(GamePathError::NotFound(game_path.to_path_buf()))
    } else if !game_path.join(GAME_EXECUTABLE).is_file() {
        Err(GamePathError::NotTheGame(game_path.to_path_buf()))
    } else {
        Ok(())
    }
}

/// Whether ReShade seems to be installed in the game folder
pub fn is_reshade_installed(game_path: &Path) -> bool {
    game_path.join("ReShade.ini").is_file()
        && RESHADE_BINARIES.iter().any(|dll| game_path.join(dll).is_file())
}

/// Looks for NieR:Automata in every Steam library of the user
///
/// # Returns
/// * [`Some`] -> The game folder
/// * [`None`] -> The game wasn't found in any known Steam library
pub fn detect_game_path() -> Option<PathBuf> {
    steam_roots()
        .into_iter()
        .flat_map(|root| steam_libraries(&root))
        .map(|library| library.join("steamapps").join("common").join(GAME_FOLDER_NAME))
        .find(|candidate| validate_game_path(candidate).is_ok())
}

/// Folders in which Steam may be installed
fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(windows)]
    {
        if let Some(path) = steam_path_from_registry() {
            roots.push(path);
        }
        for var in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Some(dir) = std::env::var_os(var) {
                roots.push(PathBuf::from(dir).join("Steam"));
            }
        }
    }

    #[cfg(not(windows))]
    if let Some(home) = dirs::home_dir() {
        roots.extend([
            home.join(".local/share/Steam"),
            home.join(".steam/steam"),
            home.join(".steam/root"),
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
            home.join("snap/steam/common/.local/share/Steam"),
        ]);
    }

    roots.retain(|r| r.is_dir());
    roots.dedup();
    roots
}

/// Reads Steam's install folder from the registry (`HKCU\Software\Valve\Steam\SteamPath`)
#[cfg(windows)]
fn steam_path_from_registry() -> Option<PathBuf> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let output = Command::new("reg")
        .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    // Output line: "    SteamPath    REG_SZ    c:/program files (x86)/steam"
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.split_once("REG_SZ"))
        .map(|(_, value)| PathBuf::from(value.trim()))
}

/// Every library folder of a Steam installation (the installation itself included)
fn steam_libraries(steam_root: &Path) -> Vec<PathBuf> {
    let mut libraries = vec![steam_root.to_path_buf()];

    if let Ok(vdf) = fs::read_to_string(steam_root.join("steamapps").join("libraryfolders.vdf")) {
        libraries.extend(parse_library_paths(&vdf));
    }

    libraries
}

/// Extracts the `"path"` values from a `libraryfolders.vdf` file
fn parse_library_paths(vdf: &str) -> Vec<PathBuf> {
    vdf.lines()
        .filter_map(|line| {
            let mut quoted = line.split('"').skip(1).step_by(2);
            match (quoted.next(), quoted.next()) {
                (Some(key), Some(value)) if key.eq_ignore_ascii_case("path") => {
                    Some(PathBuf::from(value.replace("\\\\", "\\")))
                }
                _ => None,
            }
        })
        .collect()
}

/// Errors that could occur while validating the game folder
#[derive(Error, Debug)]
pub enum GamePathError {
    /// No game path was set
    #[error("The game path isn't set. Set 'gamePath' in the settings (or use the Launcher to detect it)")]
    Unset,

    /// The game path doesn't exist
    #[error("The game path '{}' doesn't exist", .0.display())]
    NotFound(PathBuf),

    /// The game path exists but doesn't contain the game
    #[error("'{}' doesn't contain {GAME_EXECUTABLE}, it is not NieR:Automata's folder", .0.display())]
    NotTheGame(PathBuf),
}
