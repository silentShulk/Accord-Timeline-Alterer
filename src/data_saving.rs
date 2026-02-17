use std::error::Error;

use std::io::{BufReader, Write};

use std::fs::{File, create_dir_all};

use std::env::var;

use std::path::PathBuf;

use serde::{Serialize, Deserialize};



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
    pub fn load_config() -> Result<Self, Box<dyn Error>>
    {
        let home_dir = var("HOME").unwrap_or(String::from("/home/2B/"));
        let data_file_path = PathBuf::from(home_dir)
            .join(".config")
            .join("ATA")
            .join("data.json");

        if data_file_path.exists() {
            let data_file = File::open(data_file_path)?;
            let reader = BufReader::new(data_file);
            let contents = serde_json::from_reader(reader)?;

            Ok(contents)
        }
        else {
            println!("Config file (~/.config/ATA/data.json) not found, creating it with default values...\n");

            Self::create_default_config_file(data_file_path)
        }   
    }

    // creates a default config and saves it to the file
    fn create_default_config_file(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let default_config = Self::default();
        
        if let Some(data_file_folder) = path.parent() {
            create_dir_all(data_file_folder)?;
        };
        
        let mut default_config_file = File::create(path)?;

        let default_config_json = serde_json::to_string_pretty(&default_config)?;
        default_config_file.write_all(default_config_json.as_bytes())?;

        Ok(default_config)
    }
}
impl Default for Config {
    // Creates default config
    // Default game_path = $HOME(or /home/2B/)/.local/share/Steam/steamapps/common/NieRAutomata
    // Default mods = empty list
    fn default() -> Self {
        let home_dir = var("HOME").unwrap_or(String::from("/home/2B/"));
        let default_game_path = PathBuf::from(home_dir)
            .join(".local/share/Steam/steamapps/common/NieRAutomata");

        Self {
            game_path: default_game_path,
            mods: Default::default(),
        }
    }
}