use std::fs::read_dir;

use std::path::PathBuf;



// CHECK IF GIVEN PATH CONTAINS GAME FILES
pub fn check_path(current_path: &PathBuf) -> Result<bool, std::io::Error> {
    let is_gamepath = read_dir(current_path)?       
        .filter_map(|res| {     // For each entry return Some(is-exe) or None and warn the user that an entry couldn't be read
            match res {
                Ok(entry) => Some(entry.file_name() == "NieRAutomata.exe"),
                Err(e) => {
                    eprintln!("Warning: Could not read an entry in the given path: {}", e);
                    None 
                }
            }
        })
        .any(|is_match| is_match);      // Check if any of the entries that were read matched the is-exe predicate

    Ok(is_gamepath)
}



// CHECKING IF REQUIRED MODDING FILES ARE ALREADY PRESENT
pub fn check_for_required_modding_files(game_path: &PathBuf) -> Vec<PathBuf> {
    let required_files = [
        "NieRAutomata.exe",
        "d3d11.dll",
    ];

    let missing_files: Vec<PathBuf> = required_files
        .iter()
        .map(|&file| game_path.join(file))
        .filter(|path| !path.exists())
        .collect();
    
    missing_files
}