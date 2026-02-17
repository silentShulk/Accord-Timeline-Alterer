use std::error::Error;

use std::path::PathBuf;

use crate::data_saving::{Mod, ModType};

use crate::installation_utilities_and_methods::{
    check_mod_type, decompress_folder,
};
use crate::installation_utilities_and_methods::{
    install_cutscene_replacements, install_player_model, install_reshade_preset, install_texture, install_weapon_model, install_world_model,
};



/* -------------------- */
/*   MOD INSTALLATION   */
/* -------------------- */

pub fn install_mod(game_path: &PathBuf, compressed_mod_folder_path: PathBuf) -> Result<Mod, Box<dyn Error>> {
    // Check if it exists
    if !compressed_mod_folder_path.exists() {
        return Err("Mod path does not exist".into());
    }
    
    // Unzip the mod folder
    let mut mod_folder_path = decompress_folder(&compressed_mod_folder_path)?;
    
    // Get the type of mod containd
    let mod_data = check_mod_type(&mut mod_folder_path)?
       	.ok_or("The given path doesn't contain a mod")?;
    // Install the mod contained in the folder following the correct installation method
    let installed_mod = match mod_data.0 {
       	ModType::Textures => install_texture(mod_folder_path, game_path)?,
       	ModType::PlayerModels => install_player_model(mod_folder_path, game_path)?,
       	ModType::WeaponModels => install_weapon_model(mod_folder_path, game_path)?,
       	ModType::WorldModels => install_world_model(mod_folder_path, game_path)?,
        ModType::CutsceneReplacements => install_cutscene_replacements(mod_folder_path, game_path)?,
        ModType::ReshadePreset => install_reshade_preset(mod_folder_path, game_path)?,
    };
    
    Ok(installed_mod)
}



/* ---------------------- */
/*   MOD UNINSTALLATION   */
/* ---------------------- */

pub fn uninstall_mod(game_path: &PathBuf) -> Result<Mod, Box<dyn std::error::Error>> {
	Ok(Mod::new(String::from("Texture"), vec![], true, ModType::Textures))
}




pub fn list_mods(mods: &Vec<Mod>) {
    println!("Not implemented yet, pls do not kill me")
}


