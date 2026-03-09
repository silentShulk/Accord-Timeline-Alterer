use std::error::Error;

use std::path::Path;

use crate::data_config::{Mod, ModType};

use crate::installation_functions::{
    InstallationError,
    decompress_folder, ask_mod_name, get_mod_data,
    install_texture, install_player_model, install_weapon_model, install_world_model, install_cutscene_replacements, install_reshade_preset,
};
use crate::uninstallation_functions::UninstallationError;



/* -------------------- */
/*   MOD INSTALLATION   */
/* -------------------- */

pub fn install_mod(game_path: &Path, compressed_mod_folder_path: &Path) -> Result<Mod, InstallationError> {
    // Check if it exists
    if !compressed_mod_folder_path.exists() {
        return Err(InstallationError::FileAccessing(compressed_mod_folder_path.to_path_buf()));
    }
    
    let answered_name = ask_mod_name()?;
    
    // Unzip the mod folder
    let mut mod_folder_path = decompress_folder(&compressed_mod_folder_path)?;
    
    // Get the type of mod contained
    let mod_data = get_mod_data(&mut mod_folder_path)?
       	.ok_or(InstallationError::ModlessFolder(mod_folder_path.clone()))?;
    
    // Install the mod contained in the folder following the correct installation method
    let installed_mod_files = match mod_data.0 {
       	ModType::Textures => install_texture(mod_folder_path, &game_path)?,
       	ModType::PlayerModels => install_player_model(mod_folder_path, &game_path)?,
       	ModType::WeaponModels => install_weapon_model(mod_folder_path, &game_path)?,
       	ModType::WorldModels => install_world_model(mod_folder_path, &game_path)?,
        ModType::CutsceneReplacements => install_cutscene_replacements(mod_folder_path, &game_path)?,
        ModType::ReshadePreset => install_reshade_preset(&mod_folder_path, &game_path)?,
    };
    
    Ok(Mod::new(answered_name, installed_mod_files, true, mod_data.0))
}



/* ---------------------- */
/*   MOD UNINSTALLATION   */
/* ---------------------- */

pub fn uninstall_mod(game_path: &Path, installed_mods: &Vec<Mod>, mod_name: String) -> Result<Mod, UninstallationError> {
    // Check if a mod with that name exists
    let Some(mod_to_uninstall) = installed_mods.iter().find(|m| m.name == mod_name) else {
        return Err(UninstallationError::ModNotFound(mod_name))
    };
    
    // Uninstall the mod
    Ok(Mod::new(String::from(""), vec![], false, ModType::PlayerModels))
}



/* ---------------- */
/*   MOD MANAGING   */
/* ---------------- */

pub fn list_mods(mods: &Vec<Mod>) {
    println!("Not implemented yet, pls do not kill me")
}

pub fn enable_mod() {
	
}

pub fn disable_mod() {
	
}