use std::io::BufReader;

use std::fs::File;

use std::env::{VarError, var};

use std::path::PathBuf;

use thiserror::Error;

use serde::{Serialize, Deserialize};



#[derive(Error, Debug)]
pub enum ConfigLoadingError {
    #[error("The $HOME env isn't present in your system (wtf)")]
    HomeEnvNotFound(#[from] VarError),
    
    #[error("Coudln't access data.json file")]
    FileAccessing(#[from] std::io::Error),
    
    #[error("Unable to read contents of data.json")]
    JsonReading(#[from] serde_json::Error)
}



// The various types of mod that can be installed with ATA
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ModType {
    Textures,
    PlayerModels,
    WeaponModels,
    WorldModels,
    CutsceneReplacements,
    ReshadePreset,
}

// Things to take note about a mod for both mod managing and informing the user
#[derive(Serialize, Deserialize)]
pub struct Mod {
    name: String,           // Name of the mod given by the user
    files: Vec<PathBuf>,    // Files used by the mod (not the folder contaning, list of all files one by one)
    enabled: bool,          // Whether the mod is enabled or not
    mod_type: ModType,      // Type of the mod 
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
    // Save the config to file
    // fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     let data_file = File::create(DATA_FILE_PATH)?;
    //     serde_json::to_writer_pretty(data_file, self)?;
    //     Ok(())
    // }

    // Load the config from file, or load a default one
    pub fn load_config() -> Result<Self, ConfigLoadingError>
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
}