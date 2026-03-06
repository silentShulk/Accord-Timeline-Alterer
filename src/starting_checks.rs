use std::fs::read_dir;

use std::path::{Path, PathBuf};



// CHECKING IF REQUIRED MODDING FILES ARE ALREADY PRESENT
pub fn check_for_required_modding_files(game_path: &Path) -> Vec<PathBuf> {
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