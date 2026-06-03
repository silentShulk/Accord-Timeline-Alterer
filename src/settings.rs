//! **settings_management** is a module that declares Types and Functions
//! for interacting with user settings
//! 
//! In the case of ATA the user's settings are saved inside a 
//! "settings.json" file in *~/.config/ATA*

use std::{fs::File, str::FromStr};

use std::env::VarError;

use std::io::BufReader;

use std::path::PathBuf;

use thiserror::Error;

use serde::{Serialize, Deserialize};

use dirs;

use shellexpand::full;



/// Errors that could occur during interactions with the settings file
#[derive(Error, Debug)]
pub enum SettingsInteractionError {
    /// The config directory cannot be determined (e.g. $HOME not set)
    #[error("Couldn't determine the config directory. {0}")]
    ConfigDirNotFound(#[from] VarError),

    /// The settings.json file in *~/.config/ATA* could not be accessed
    #[error("Couldn't access settings file (settings.json found inside config dir of OS). {0}")]
    SettingsFileAccessing(#[from] std::io::Error),

    /// The contents of settings.json were impossible to read
    #[error("Unable to read contents of settings file (settings.json found inside config dir of OS). {0}")]
    JsonReading(#[from] serde_json::Error),

    #[error("Unable to parse received setting name ({0}) into and actual setting")]
    InvalidSettingName(String),

    #[error("Unable to parse received setting value ({0}) into a value acceptable for the given setting")]
    InvalidSettingValue(String),
}



/// The visual style / layout theme of the UI
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum Style {
    #[default]
    SilentShulk,
    Beyluta,
}
impl FromStr for Style {
    type Err = SettingsInteractionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SilentShulk" => Ok(Self::SilentShulk),
            "Beyluta" => Ok(Self::Beyluta),
            _ => Err(SettingsInteractionError::InvalidSettingValue(s.to_string())),
        }
    }
}

/// The color palette applied to the UI
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum Palette {
    #[default]
    Automata,
    Replicant,
}
impl FromStr for Palette {
    type Err = SettingsInteractionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Automata" => Ok(Self::Automata),
            "Replicant" => Ok(Self::Replicant),
            _ => Err(SettingsInteractionError::InvalidSettingValue(s.to_string())),
        }
    }
}


/// The order in which installed mods are displayed
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum SortingOrder {
    #[default]
    ModType,
    EnableStatus,
    Alphabetical,
    InstallDate,
    Size,
}
impl FromStr for SortingOrder {
    type Err = SettingsInteractionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ModType" => Ok(Self::ModType),
            "EnableStatus" => Ok(Self::EnableStatus),
            "Alphabetical" => Ok(Self::Alphabetical),
            "InstallDate" => Ok(Self::InstallDate),
            "Size" => Ok(Self::Size),
            _ => Err(SettingsInteractionError::InvalidSettingValue(s.to_string())),
        }
    }
}


/// What ATA does when a mod file would overwrite a file already in the game folder
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum ConflictResolution {
    #[default]
    Ask,
    Overwrite,
    Skip,
}
impl FromStr for ConflictResolution {
    type Err = SettingsInteractionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Ask" => Ok(Self::Ask),
            "Overwrite" => Ok(Self::Overwrite),
            "Skip" => Ok(Self::Skip),
            _ => Err(SettingsInteractionError::InvalidSettingValue(s.to_string())),
        }
    }
}



/// All user-configurable settings for ATA
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Visual style / layout theme
    pub style: Style,
    /// Color palette
    pub palette: Palette,
    /// Order in which mods are shown in the list
    pub sorting_order: SortingOrder,

    /// How to handle file conflicts during installation
    pub files_conflict_resolution: ConflictResolution,
    /// Whether to keep the extracted temp folder after installation
    pub keep_extracted_folders: bool,
    /// Path to the extracted folders location
    pub extracted_folders_location: PathBuf,

    /// Path to the game's installation folder 
    pub game_path: PathBuf,
    /// Discord Rich Presence application ID (empty string = disabled)
    pub discord_rich_presence: String,
}

impl Settings {
    /// Creates a Settings instance from the settings file (*~/.config/ATA/settings.json*)
    /// 
    /// # Errors
    /// * [`SettingsInteractionError::ConfigDirNotFound`] if the config directory cannot be determined
    /// * [`SettingsInteractionError::SettingsFileAccessing`] if the settings file cannot be accessed
    /// * [`SettingsInteractionError::JsonReading`] if the settings file cannot be parsed
    pub fn load_settings() -> Result<Self, SettingsInteractionError> {
        let config_dir = dirs::config_dir()
            .ok_or(SettingsInteractionError::ConfigDirNotFound(VarError::NotPresent))?;
        let settings_file_path = PathBuf::from(config_dir)
            .join("ATA")
            .join("settings.json");

        let settings_file = File::open(settings_file_path)?;
        let reader = BufReader::new(settings_file);
        let mut contents: Settings = serde_json::from_reader(reader)?;

        // Expand shell variables / ~ in the game path (e.g. "~/Games/NieR")
        let path_str = contents.game_path.to_string_lossy().into_owned();
        let expanded = full(&path_str)
            .map_err(|_| SettingsInteractionError::ConfigDirNotFound(VarError::NotPresent))?;
        contents.game_path = PathBuf::from(expanded.as_ref());

        Ok(contents)
    }
    
    /// Saves the current settings to the settings file (*~/.config/ATA/settings.json*)
    /// 
    /// # Errors
    /// * [`SettingsInteractionError::ConfigDirNotFound`] if the config directory cannot be determined
    /// * [`SettingsInteractionError::SettingsFileAccessing`] if the settings file cannot be written
    /// * [`SettingsInteractionError::JsonReading`] if the settings cannot be serialized
    pub fn update_settings_file(&self) -> Result<(), SettingsInteractionError> {
        let config_dir = dirs::config_dir()
            .ok_or(SettingsInteractionError::ConfigDirNotFound(VarError::NotPresent))?;
        let settings_file_path = PathBuf::from(config_dir)
            .join("ATA")
            .join("settings.json");

        let settings_file = File::create(settings_file_path)?;
        serde_json::to_writer_pretty(settings_file, &self)?;

        Ok(())
    }

    /// Recognizes the setting that needs to be changed and updates it
    /// 
    pub fn update_setting(&mut self, setting: String, value: String) -> Result<Settings, SettingsInteractionError> {
        match setting.as_str() {
            "style" => self.style = value.parse::<Style>()?,
            "palette" => self.palette = value.parse::<Palette>()?,
            "sortingOrder" => self.sorting_order = value.parse::<SortingOrder>()?,
            "filesConflictResolution" => self.files_conflict_resolution = value.parse::<ConflictResolution>()?,
            "keepExtractedFolders" => self.keep_extracted_folders = value.parse::<bool>().map_err(|_| SettingsInteractionError::InvalidSettingName(setting))?,
            "extractedFoldersLocation" => self.extracted_folders_location = value.parse::<PathBuf>().map_err(|_| SettingsInteractionError::InvalidSettingName(setting))?,
            "gamePath" => self.game_path = value.parse::<PathBuf>().map_err(|_| SettingsInteractionError::InvalidSettingName(setting))?,
            "discordRichPresence" => self.discord_rich_presence = value,
            _ => return Err(SettingsInteractionError::InvalidSettingName(setting)),
        };

        self.update_settings_file()?;

        Ok(self.clone())
    }
}