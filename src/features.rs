use std::fs::rename;

use std::path::{PathBuf, Path};

use crate::data_config::{Mod, ModType};

use crate::installation_utilities::{
    InstallationError,
    decompress_folder, ask_mod_name, get_mod_data,
    install_texture, install_player_model, install_weapon_model, install_world_model, install_cutscene_replacements, install_reshade_preset,
};
use crate::uninstallation_utilities::{
    UninstallationError, remove_mod_files
};
use crate::enabling_disabling_utilities::*;



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

pub fn uninstall_mod(installed_mods: &Vec<Mod>, mod_name: String) -> Result<usize, UninstallationError> {
    let index_to_uninstall = installed_mods
        .iter()
        .position(|m| m.name == mod_name)
        .ok_or(UninstallationError::ModNotFound(mod_name))?;

    remove_mod_files(installed_mods[index_to_uninstall].clone().files)?;
    
    Ok(index_to_uninstall)
}



/* ---------------- */
/*   MOD MANAGING   */
/* ---------------- */

pub fn list_mods(mods: &Vec<Mod>) {
    println!("List of mods:\n");
    for installed_mod in mods {
        println!("\t- {}\n", installed_mod.name);
        
        println!("\t\tFiles: ");
        for file in installed_mod.files.clone() {
            println!("\t\t- {:?}", file)
        }
        
        println!("Enabled: {}", if installed_mod.enabled==true {"Yes"} else {"No"});
        
        println!("Mod Type: {}", installed_mod.mod_type)
    }
}

pub fn enable_mod(game_path: &PathBuf, mod_to_enable: &Mod) -> Result<(), EnablingDisablingError>  {
    for file in &mod_to_enable.files {
        let Some(file_name) = file.file_name() else {
            return Err(EnablingDisablingError::DotDotPath(file.to_path_buf()))
        };
        let Some(file_parent) = file.parent() else {
            return Err(EnablingDisablingError::ParentlessOrEmptyPath(file.to_path_buf()))
        };
        
        rename(file, file_parent.join(file_name))?;
    }
    
    Ok(())
}

pub fn disable_mod(game_path: &PathBuf, mod_to_disable: &Mod) -> Result<(), EnablingDisablingError>  {
    for file in &mod_to_disable.files {
        let Some(file_name) = file.file_name() else {
            return Err(EnablingDisablingError::DotDotPath(file.to_path_buf()))
        };
        let Some(file_parent) = file.parent() else {
            return Err(EnablingDisablingError::ParentlessOrEmptyPath(file.to_path_buf()))
        };
        
        rename(file, file_parent.join(".disabled").join(file_name))?;
    }
    
    Ok(())
}