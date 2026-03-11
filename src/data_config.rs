use std::fs::File;

use std::env::{VarError, var};

use std::io::BufReader;

use std::path::PathBuf;

use std::fmt;

use thiserror::Error;

use serde::{Serialize, Deserialize};



#[derive(Error, Debug)]
pub enum ConfigInteractionError {
    #[error("The $HOME env isn't present in your system (wtf)")]
    HomeEnvNotFound(#[from] VarError),
    
    #[error("Coudln't access data.json file")]
    DataFileAccessing(#[from] std::io::Error),
    
    #[error("Unable to read contents of data.json")]
    JsonReading(#[from] serde_json::Error)
}



// The various types of mod that can be installed with ATA
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub enum ModType {
    Textures,
    PlayerModels,
    WeaponModels,
    WorldModels,
    CutsceneReplacements,
    ReshadePreset,
}
impl fmt::Display for ModType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModType::Textures => write!(f, "Textures"),
            ModType::PlayerModels => write!(f, "Player Models"),
            ModType::WeaponModels => write!(f, "Weapon Models"),
            ModType::WorldModels => write!(f, "World Models"),
            ModType::CutsceneReplacements => write!(f, "Cutscene Replacements"),
            ModType::ReshadePreset => write!(f, "ReShade Preset"),
        }
    }
}

// Things to take note about a mod for both mod managing and informing the user
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Mod {
    pub name: String,           // Name of the mod given by the user
    pub files: Vec<PathBuf>,    // Files used by the mod (not the folder contaning, list of all files one by one)
    pub enabled: bool,          // Whether the mod is enabled or not
    pub mod_type: ModType,      // Type of the mod 
}
impl Mod {
    pub fn new(name: String, files: Vec<PathBuf>, enabled: bool, mod_type: ModType) -> Self {
        Self {
            name,
            files,
            enabled,
            mod_type,
        }
    }
}

// What to save in the data file
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub game_path: PathBuf,
    pub mods: Vec<Mod>,
}
impl Config {
	// Load the config from file, or load a default one
    pub fn load_config() -> Result<Self, ConfigInteractionError>
    {
        let home_dir = var("HOME")?;
        let data_file_path = PathBuf::from(home_dir)
            .join(".config")
            .join("ATA")
            .join("data.json");

        let data_file = File::open(data_file_path)?;
        let reader = BufReader::new(data_file);
        let contents = serde_json::from_reader(reader)?;

        Ok(contents)
    }
	
	pub fn save_new_mod(&mut self, new_mod: Mod) -> Result<(), ConfigInteractionError>{
		self.mods.push(new_mod);
		
		self.update_data_file()
	}
	
	pub fn remove_mod(&mut self, index_to_remove: usize) -> Result<(), ConfigInteractionError> {
        self.mods.remove(index_to_remove);

		self.update_data_file()
	}       

    fn update_data_file(&self) -> Result<(), ConfigInteractionError> {
    	let home_dir = var("HOME")?;
     	let data_file_path = PathBuf::from(home_dir)
        	.join(".config")
         	.join("ATA")
          	.join("data.json");
    
    	let data_file = File::open(data_file_path)?;
    	serde_json::to_writer_pretty(data_file, &self)?;
     
     	Ok(())
    }
}