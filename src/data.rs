//! **data_management** is a module that declares Types and Functions
//! for interacting with saved data
//! 
//! In the case of ATA the saved data is stored inside a 
//! "data.json" file in *~/.local/share/ATA*

use std::fs::File;

use std::env::VarError;

use std::io::BufReader;

use std::path::PathBuf;

use std::fmt::{self};

use thiserror::Error;

use serde::{Serialize, Deserialize};

use dirs;

use chrono::DateTime;
use chrono::Utc;

use shellexpand::full;



/// Errors that could occur during interactions with the saved data
#[derive(Error, Debug)]
pub enum DataInteractionError {
    /// The $HOME environment variable is not present in the system
    /// 
    /// I have no idea how this could possibly happen on a working Linux installation
    #[error("The $HOME env isn't present in your system (wtf). {0}")]
    HomeEnvNotFound(#[from] VarError),
    
    /// The data.json file in *~/.local/share/ATA* could not be accessed
    /// 
    /// It could either be absent, have had its name changed or have gotten corrupted
    #[error("Couldn't access data file (data.json found inside data dir of OS). {0}")]
    DataFileAccessing(#[from] std::io::Error),
    
    /// The contents of data.json were impossible to read
    /// 
    /// This could be because either the file is corrupted or contains invalid JSON
    #[error("Unable to read contents of data file (data.json found inside data dir of OS). {0}")]
    JsonReading(#[from] serde_json::Error),

    #[error("The mod name '{0}' already exists")]
    ModNameExists(String),
}



/// Mod types supported by ATA
///
/// Mod types not currently supported are not generic, but mod-specific (like NAIOM)
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ModType {
    /// `DLL` mods are unique mods
    /// They contain **dll** files and other files
    DLL,
    /// `Textures` mods contain textures for models
    /// They contain **dds** files
    Textures,
    /// `Player models` mods contain 3D models for 2B, 9S, A2
    /// They contain **dtt/dat** files
    PlayerModels,
    /// `Weapon models` mods contain 3D models for weapons
    /// They contain **dtt/dat** files
    WeaponModels,
    /// `World models` mods contain 3D models for world objects
    /// They contain **dtt/dat** files
    WorldModels,
    /// `Cutscene replacements` mods contain replacement for the game's cutscenes
    /// They contain **usm** files
    CutsceneReplacements,
    /// `Reshade presets` mods contain shader presets
    /// They contain **ini** files and other files
    ReshadePreset,
}
impl ModType {
    /// Returns the folder in which that mod type's files are installed
    ///
    /// # Returns
    /// * *SK_Res/inject/textures/* for textures
    /// * *data/pl/* for player models
    /// * *data/wp/* for weapon models
    /// * *data/bg/* for world models
    /// * *data/movie/* for cutscene replacements
    /// * *idk* for reshade presets
    pub fn get_corresponding_folder(&self) -> String {
        match self {
            ModType::DLL => String::from(""),
            ModType::Textures => String::from("SK_Res/inject/textures/"),
            ModType::PlayerModels => String::from("data/pl/"),
            ModType::WeaponModels => String::from("data/wp/"),
            ModType::WorldModels => String::from("data/bg/"),
            ModType::CutsceneReplacements => String::from("data/movie/"),
            ModType::ReshadePreset => String::from("idk"),
        }
    }

    /// Returns a short ID for the ModType, usually the starting letters of each word.
    /// 
    /// # Returns
    /// A string slice containing the ID.
    /// * `DLL` -> `"DLL"`
    /// * `Textures` -> `"T"`
    /// * `PlayerModels` -> `"PL"`
    /// * `WeaponModels` -> `"WeM"`
    /// * `WorldModels` -> `"WoM"`
    /// * `CutsceneReplacements` -> `"CR"`
    /// * `ReshadePreset` -> `"RP"`
    fn get_id(&self) -> &str {
        match self {
            ModType::DLL => "DLL",
            ModType::Textures => "T",
            ModType::PlayerModels => "PL",
            ModType::WeaponModels => "WeM",
            ModType::WorldModels => "WoM",
            ModType::CutsceneReplacements => "CR",
            ModType::ReshadePreset => "RP",
        }
    }
}
impl fmt::Display for ModType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModType::DLL => write!(f, "Unique mod"),
            ModType::Textures => write!(f, "Textures"),
            ModType::PlayerModels => write!(f, "Player Models"),
            ModType::WeaponModels => write!(f, "Weapon Models"),
            ModType::WorldModels => write!(f, "World Models"),
            ModType::CutsceneReplacements => write!(f, "Cutscene Replacements"),
            ModType::ReshadePreset => write!(f, "ReShade Preset"),
        }
    }
}

/// Things to take note about a mod for both mod managing and informing the user
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Mod {
    /// Name of the mod given by the user
    pub name: String,
    /// Files used by the mod (not the folder containing, list of all files one by one)
    pub files: Vec<PathBuf>,
    /// Whether the mod is enabled or not
    pub enabled: bool,
    /// Type of the mod
    pub mod_type: ModType,
    /// Date and time the mod was installed
    pub install_date: DateTime<Utc>,
    /// Unique identifier for the mod
    pub uid: String,
}
impl Mod {
    /// Creates a new mod with the given name, files, enabled status, mod type, date of installation and UID
    /// 
    /// # Arguments
    /// * `name` - Name of the mod given by the user
    /// * `files` - Files used by the mod (not the folder containing, list of all files one by one)
    /// * `enabled` - Whether the mod is enabled or not
    /// * `mod_type` - Type of the mod
    /// * `install_date` - Date and time the mod was installed
    pub fn new(name: String, files: Vec<PathBuf>, enabled: bool, mod_type: ModType, install_date: DateTime<Utc>) -> Self {
        Self {
            uid: Self::get_uid(&name, &mod_type, &install_date),
            name,
            files,
            enabled,
            mod_type,
            install_date,
        }
    }

    /// Returns the UID of the mod based on its name, type and installation date
    /// 
    /// # Arguments
    /// * `mod_name` - Name of the mod given by the user
    /// * `mod_type` - Type of the mod
    /// * `install_date` - Date and time the mod was installed
    /// 
    /// # Returns
    /// A String containing the UID of the mod
    fn get_uid(mod_name: &str, mod_type: &ModType, install_date: &DateTime<Utc>) -> String {
        let name = &mod_name[0..1];
        let m_type = mod_type.get_id();
        let date = install_date.format("%Y%m%d%H%M%S").to_string();

        format!("{}{}{}", name, m_type, date)
    }
}
impl fmt::Display for Mod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mod_description = format!(
            "Name: {}\n\tFiles location: {:?}\n\tEnabled? {}\n\tMod Type: {}",
            self.name,
            self.mod_type.get_corresponding_folder(),
            if self.enabled { "Yes" } else { "No" },
            self.mod_type,
        );
        write!(f, "{}", mod_description)
    }
}

/// Holds the runtime state of ATA: the list of installed mods.
#[derive(Serialize, Deserialize)]
pub struct Data {
    /// List of mods installed
    pub mods: Vec<Mod>,
}
impl Data {
    /// Creates a Data instance from the data file (*~/.local/share/ATA/data.json*)
    /// 
    /// # Errors
    /// * [`DataInteractionError::HomeEnvNotFound`] if the data directory cannot be determined
    /// * [`DataInteractionError::DataFileAccessing`] if the data file cannot be accessed
    /// * [`DataInteractionError::JsonReading`] if the data file cannot be parsed
    pub fn load_data() -> Result<Self, DataInteractionError> {
        let data_dir = dirs::data_dir()
            .ok_or(DataInteractionError::HomeEnvNotFound(VarError::NotPresent))?;
        let data_file_path = PathBuf::from(data_dir)
            .join("ATA")
            .join("data.json");

        let data_file = File::open(data_file_path)?;
        let reader = BufReader::new(data_file);
        let mut contents: Data = serde_json::from_reader(reader)?;

        for m in &mut contents.mods {
            m.files = m.files.iter().map(|f| {
                let s = f.to_string_lossy().into_owned();
                PathBuf::from(full(&s).map(|e| e.into_owned()).unwrap_or(s))
            }).collect();
        }

        Ok(contents)
    }

    /// Returns `Ok(name)` if the name is not already taken, `Err` otherwise
    pub fn name_exists(&self, name: &str) -> Result<(), DataInteractionError> {
        if self.mods.iter().any(|m| m.name == name) {
            Err(DataInteractionError::ModNameExists(name.to_owned()))
        } else {
            Ok(())
        }
    }

    /// Saves a new mod to the Data struct and writes the data file
    /// 
    /// # Errors
    /// * [`DataInteractionError::HomeEnvNotFound`] if the data directory cannot be determined
    /// * [`DataInteractionError::DataFileAccessing`] if the data file cannot be accessed
    /// * [`DataInteractionError::JsonReading`] if the data file cannot be serialized
    pub fn save_new_mod(&mut self, new_mod: &Mod) -> Result<(), DataInteractionError> {
        self.mods.push(new_mod.clone());
        self.update_data_file()
    }

    /// Returns the index and a clone of the mod with the given name, if it exists
    pub fn get_mod_by_name(&self, name: &str) -> Option<(usize, Mod)> {
        self.mods.iter().enumerate()
            .find(|(_, m)| m.name == name)
            .map(|(i, m)| (i, m.clone()))
    }

    /// Removes the mod at `index_to_remove` and writes the data file
    /// 
    /// # Errors
    /// * [`DataInteractionError::HomeEnvNotFound`] if the data directory cannot be determined
    /// * [`DataInteractionError::DataFileAccessing`] if the data file cannot be accessed
    /// * [`DataInteractionError::JsonReading`] if the data file cannot be serialized
    pub fn remove_mod(&mut self, index_to_remove: usize) -> Result<(), DataInteractionError> {
        self.mods.remove(index_to_remove);
        self.update_data_file()
    }

    /// Toggles the enabled state of the mod at `index` and updates its file list, then writes the data file
    /// 
    /// # Errors
    /// * [`DataInteractionError::HomeEnvNotFound`] if the data directory cannot be determined
    /// * [`DataInteractionError::DataFileAccessing`] if the data file cannot be accessed
    /// * [`DataInteractionError::JsonReading`] if the data file cannot be serialized
    pub fn switch_mod_state(&mut self, index: usize, new_files: Vec<PathBuf>) -> Result<(), DataInteractionError> {
        self.mods[index].files = new_files;
        self.mods[index].enabled = !self.mods[index].enabled;
        self.update_data_file()
    }

    /// Rewrites the data file with the current in-memory state
    fn update_data_file(&self) -> Result<(), DataInteractionError> {
        let data_dir = dirs::data_dir()
            .ok_or(DataInteractionError::HomeEnvNotFound(VarError::NotPresent))?;
        let data_file_path = PathBuf::from(data_dir)
            .join("ATA")
            .join("data.json");

        let data_file = File::create(data_file_path)?;
        serde_json::to_writer_pretty(data_file, &self)?;

        Ok(())
    }
}
