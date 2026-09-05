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

use crate::paths::PATHS;
use crate::utils::files::{expand_path, FilesInteractionError};
use crate::utils::json::{JsonParsingError, load_json, save_json};

use std::path::PathBuf;
use std::str::FromStr;

use thiserror::Error;
use serde::{Deserialize, Serialize};



/// All user-configurable settings for ATA
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Visual layout theme applied to the UI
    pub style: String,
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
    /// Also expands any shell variables or `~` present in `game_path` and `extracted_folders_location`.
    ///
    /// # Returns
    /// * [`Ok`] -> A [`Settings`] instance populated from the settings file
    /// * [`Err`] -> The type of error that occurred
    ///
    /// # Errors
    /// * [`SettingsInteractionError::FilesInteraction`] if a shell variable in a path cannot be resolved
    /// * [`SettingsInteractionError::Json`] if the settings file cannot be opened, read, or parsed
    pub fn load_settings() -> Result<Self, SettingsInteractionError> {
        let mut contents: Settings = load_json(&PATHS.settings_file)?;

        for path in [&mut contents.game_path, &mut contents.extracted_folders_location] {
            *path = expand_path(&path.to_string_lossy())?;
        }

        Ok(contents)
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
    /// * [`SettingsInteractionError::FilesInteraction`] if a shell variable in a path value cannot be resolved
    /// * [`SettingsInteractionError::Json`] if the settings file cannot be written
    pub fn update_setting(
        &mut self,
        setting: String,
        value: String,
    ) -> Result<Settings, SettingsInteractionError> {
        match setting.as_str() {
            "style" => self.style = value,
            "palette" => self.palette = value.parse::<Palette>()?,
            "sortingOrder" => self.sorting_order = value.parse::<SortingOrder>()?,
            "filesConflictResolution" => {
                self.files_conflict_resolution = value.parse::<ConflictResolution>()?
            }
            "keepExtractedFolders" => {
                self.keep_extracted_folders = value
                    .parse::<bool>()
                    .map_err(|_| SettingsInteractionError::InvalidSettingValue(value.clone()))?
            }
            "extractedFoldersLocation" => self.extracted_folders_location = expand_path(&value)?,
            "gamePath" => self.game_path = expand_path(&value)?,
            "discordRichPresence" => self.discord_rich_presence = value,
            _ => return Err(SettingsInteractionError::InvalidSettingName(setting)),
        };

        save_json(&PATHS.settings_file, &self)?;

        Ok(self.clone())
    }
}

/// Errors that could occur during interactions with the settings file
#[derive(Error, Debug)]
pub enum SettingsInteractionError {
    /// The provided setting name does not correspond to any known setting
    #[error("Unable to parse received setting name ({0}) into and actual setting")]
    InvalidSettingName(String),

    /// The provided value cannot be parsed into the type required by the target setting
    #[error(
        "Unable to parse received setting value ({0}) into a value acceptable for the given setting"
    )]
    InvalidSettingValue(String),

    /// Reading or writing the settings file as JSON failed
    #[error("Could'not parse Json for saving/loading settings. {0}")]
    Json(#[from] JsonParsingError),

    /// A path component of a setting value could not be extracted or expanded
    #[error("An error occurred while interacting with files. {0}")]
    FilesInteraction(#[from] FilesInteractionError),
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
    /// Sort mods from newest to oldest install date
    InstallDate,
    /// Sort enabled mods before disabled ones
    EnableStatus,
    /// Sort mods alphabetically by name
    Alphabetical,
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
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default, Copy)]
pub enum ConflictResolution {
    /// Prompt the user to decide for each conflicting file (default)
    #[default]
    Warn,
    /// Silently overwrite the existing file with the mod's version
    Overwrite,
}

impl FromStr for ConflictResolution {
    type Err = SettingsInteractionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Warn" => Ok(Self::Warn),
            "Overwrite" => Ok(Self::Overwrite),
            _ => Err(SettingsInteractionError::InvalidSettingValue(s.to_string())),
        }
    }
}