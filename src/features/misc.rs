//! **misc** is a module that contains miscellaneous functions for secondary processes
//!
//! This includes:
//! * **launching**: Starting NieR:Automata using Steam URI invocation
//!
//! API functions: [`launch_automata`]

use std::process::Command;

/// Launches NieR:Automata via Steam protocol standard (`steam://run/524220`)
///
/// Uses `xdg-open` on Linux targets and `cmd /C start` on Windows targets.
///
/// # Returns
/// * [`Ok`] -> `()` on successfully spawning the process launcher command
/// * [`Err`] -> [`std::io::Error`] if execution fails
///
/// # Errors
/// * Returns [`std::io::Error`] if the underlying OS process launcher cannot be executed
pub fn launch_automata() -> Result<(), std::io::Error> {
    let app_id = "524220";
    let steam_url = format!("steam://run/{}", app_id);

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&steam_url)
            .spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", &steam_url])
            .spawn()?;
    }

    Ok(())
}