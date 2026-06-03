//! **settings** is a module that declares types and functions
//! for interacting with user settings
//!
//! In the case of ATA the user's settings are saved inside a
//! "settings.json" file in *~/.config/ATA*
//!
//! This includes:
//! * **loading**: Reading the settings file into a [`Settings`] struct
//! * **saving**: Writing the current in-memory settings back to the settings file
//! * **updating**: Parsing and applying a single setting change by name and value
//!
//! Main type: [`Settings`]

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
    /// The config directory cannot be determined (e.g. `$HOME` not set)
    #[error("Couldn't determine the config directory. {0}")]
    ConfigDirNotFound(#[from] VarError),

    /// The settings.json file in *~/.config/ATA* could not be accessed
    ///
    /// It could either be absent, have had its name changed, or have gotten corrupted
    #[error("Couldn't access settings file (settings.json found inside config dir of OS). {0}")]
    SettingsFileAccessing(#[from] std::io::Error),

    /// The contents of settings.json were impossible to read
    ///
    /// This could be because the file is corrupted or contains invalid JSON
    #[error("Unable to read contents of settings file (settings.json found inside config dir of OS). {0}")]
    JsonReading(#[from] serde_json::Error),

    /// The provided setting name does not correspond to any known setting
    #[error("Unable to parse received setting name ({0}) into and actual setting")]
    InvalidSettingName(String),

    /// The provided value cannot be parsed into the type required by the target setting
    #[error("Unable to parse received setting value ({0}) into a value acceptable for the given setting")]
    InvalidSettingValue(String),
}



/// The visual layout theme applied to the UI
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum Style {
    /// Default style, designed by silentShulk
    #[default]
    SilentShulk,
    /// Alternative style, designed by Beyluta
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
    /// Default palette, inspired by NieR: Automata
    #[default]
    Automata,
    /// Alternative palette, inspired by NieR: Replicant
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


/// The order in which installed mods are displayed in the list view
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum SortingOrder {
    /// Group mods by their [`crate::data::ModType`] (default)
    #[default]
    ModType,
    /// Sort enabled mods before disabled ones
    EnableStatus,
    /// Sort mods alphabetically by name
    Alphabetical,
    /// Sort mods from newest to oldest install date
    InstallDate,
    /// Sort mods from largest to smallest total file size
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


/// What ATA does when a mod file would overwrite a file already present in the game folder
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum ConflictResolution {
    /// Prompt the user to decide for each conflicting file (default)
    #[default]
    Ask,
    /// Silently overwrite the existing file with the mod's version
    Overwrite,
    /// Silently leave the existing file in place and skip the mod's version
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
    /// Visual layout theme applied to the UI
    pub style: Style,
    /// Color palette applied to the UI
    pub palette: Palette,
    /// Order in which mods are shown in the list view
    pub sorting_order: SortingOrder,

    /// How to handle file conflicts when installing a mod
    pub files_conflict_resolution: ConflictResolution,
    /// Whether to keep the extracted temporary folder after a successful installation
    pub keep_extracted_folders: bool,
    /// Filesystem path where extracted temporary folders are stored
    pub extracted_folders_location: PathBuf,

    /// Absolute path to the game's installation folder
    pub game_path: PathBuf,
    /// Discord Rich Presence application ID; empty string means Rich Presence is disabled
    pub discord_rich_presence: String,
}

impl Settings {
    /// Creates a [`Settings`] instance from the settings file (*~/.config/ATA/settings.json*)
    ///
    /// Also expands any shell variables or `~` present in `game_path`.
    ///
    /// # Returns
    /// * [`Ok`] -> A [`Settings`] instance populated from the settings file
    /// * [`Err`] -> The type of error that occurred
    ///
    /// # Errors
    /// * [`SettingsInteractionError::ConfigDirNotFound`] if the config directory cannot be determined
    /// * [`SettingsInteractionError::SettingsFileAccessing`] if the settings file cannot be opened
    /// * [`SettingsInteractionError::JsonReading`] if the settings file cannot be parsed as JSON
    pub fn load_settings() -> Result<Self, SettingsInteractionError> {
        let settings_file = File::open(settings_file_path()?)?;
        let reader = BufReader::new(settings_file);
        let mut contents: Settings = serde_json::from_reader(reader)?;

        // Expand shell variables / ~ in the game path (e.g. "~/Games/NieR")
        let path_str = contents.game_path.to_string_lossy().into_owned();
        let expanded = full(&path_str)
            .map_err(|_| SettingsInteractionError::ConfigDirNotFound(VarError::NotPresent))?;
        contents.game_path = PathBuf::from(expanded.as_ref());

        Ok(contents)
    }

    /// Overwrites the settings file with the current in-memory state
    ///
    /// # Returns
    /// * [`Ok`] -> `()` on success
    /// * [`Err`] -> The type of error that occurred
    ///
    /// # Errors
    /// * [`SettingsInteractionError::ConfigDirNotFound`] if the config directory cannot be determined
    /// * [`SettingsInteractionError::SettingsFileAccessing`] if the settings file cannot be created or written
    /// * [`SettingsInteractionError::JsonReading`] if the settings cannot be serialized
    pub fn update_settings_file(&self) -> Result<(), SettingsInteractionError> {
        let settings_file = File::create(settings_file_path()?)?;
        serde_json::to_writer_pretty(settings_file, &self)?;

        Ok(())
    }

    /// Parses `value`, applies it to the setting identified by `setting`, and persists the change
    ///
    /// Setting names use camelCase and mirror the JSON keys in settings.json
    /// (e.g. `"sortingOrder"`, `"gamePath"`).
    ///
    /// # Arguments
    /// * `setting` - camelCase name of the setting to update
    /// * `value` - String representation of the new value
    ///
    /// # Returns
    /// * [`Ok`] -> A clone of the updated [`Settings`] struct
    /// * [`Err`] -> The type of error that occurred
    ///
    /// # Errors
    /// * [`SettingsInteractionError::InvalidSettingName`] if `setting` does not match any known setting
    /// * [`SettingsInteractionError::InvalidSettingValue`] if `value` cannot be parsed for the target setting
    /// * [`SettingsInteractionError::ConfigDirNotFound`] if the config directory cannot be determined
    /// * [`SettingsInteractionError::SettingsFileAccessing`] if the settings file cannot be written
    /// * [`SettingsInteractionError::JsonReading`] if the updated settings cannot be serialized
    pub fn update_setting(&mut self, setting: String, value: String) -> Result<Settings, SettingsInteractionError> {
        match setting.as_str() {
            "style" => self.style = value.parse::<Style>()?,
            "palette" => self.palette = value.parse::<Palette>()?,
            "sortingOrder" => self.sorting_order = value.parse::<SortingOrder>()?,
            "filesConflictResolution" => self.files_conflict_resolution = value.parse::<ConflictResolution>()?,
            "keepExtractedFolders" => self.keep_extracted_folders = value.parse::<bool>().map_err(|_| SettingsInteractionError::InvalidSettingValue(value.clone()))?,
            "extractedFoldersLocation" => self.extracted_folders_location = value.parse::<PathBuf>().map_err(|_| SettingsInteractionError::InvalidSettingValue(value.clone()))?,
            "gamePath" => self.game_path = value.parse::<PathBuf>().map_err(|_| SettingsInteractionError::InvalidSettingValue(value.clone()))?,
            "discordRichPresence" => self.discord_rich_presence = value,
            _ => return Err(SettingsInteractionError::InvalidSettingName(setting)),
        };

        self.update_settings_file()?;

        Ok(self.clone())
    }
}



/// Returns the canonical path to *~/.config/ATA/settings.json*
///
/// Centralised so that [`Settings::load_settings`] and [`Settings::update_settings_file`]
/// never diverge in where they look for the file.
///
/// # Returns
/// * [`Ok`] -> The resolved [`PathBuf`]
/// * [`Err`] -> [`SettingsInteractionError::ConfigDirNotFound`] if the config directory cannot be determined
fn settings_file_path() -> Result<PathBuf, SettingsInteractionError> {
    let config_dir = dirs::config_dir()
        .ok_or(SettingsInteractionError::ConfigDirNotFound(VarError::NotPresent))?;
    Ok(PathBuf::from(config_dir).join("ATA").join("settings.json"))
}
