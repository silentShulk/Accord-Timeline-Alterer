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

use std::{fs::File, str::FromStr};

use std::env::VarError;

use std::io::BufReader;

use std::path::PathBuf;

use thiserror::Error;

use serde::{Deserialize, Serialize};

use shellexpand::full;



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
    /// * [`SettingsInteractionError::EnvExpansion`] if a shell variable in a path cannot be resolved
    /// * [`SettingsInteractionError::SettingsFileAccessing`] if the settings file cannot be opened
    /// * [`SettingsInteractionError::JsonReading`] if the settings file cannot be parsed as JSON
    pub fn load_settings() -> Result<Self, SettingsInteractionError> {
        let settings_file = File::open(&PATHS.settings_file)?;
        let reader = BufReader::new(settings_file);
        let mut contents: Settings = serde_json::from_reader(reader)?;

        for path in [
            &mut contents.game_path,
            &mut contents.extracted_folders_location,
        ] {
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
    /// * [`SettingsInteractionError::EnvExpansion`] if a shell variable in a path value cannot be resolved
    /// * [`SettingsInteractionError::SettingsFileAccessing`] if the settings file cannot be written
    /// * [`SettingsInteractionError::JsonReading`] if the updated settings cannot be serialized
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

        self.update_settings_file()?;

        Ok(self.clone())
    }

    /// Overwrites the settings file with the current in-memory state
    ///
    /// # Returns
    /// * [`Ok`] -> `()` on success
    /// * [`Err`] -> The type of error that occurred
    ///
    /// # Errors
    /// * [`SettingsInteractionError::SettingsFileAccessing`] if the settings file cannot be created or written
    /// * [`SettingsInteractionError::JsonReading`] if the settings cannot be serialized
    pub fn update_settings_file(&self) -> Result<(), SettingsInteractionError> {
        let settings_file = File::create(&PATHS.settings_file)?;
        serde_json::to_writer_pretty(settings_file, &self)?;

        Ok(())
    }
}



/// Errors that could occur during interactions with the settings file
#[derive(Error, Debug)]
pub enum SettingsInteractionError {
    /// The config directory cannot be determined (e.g. `$HOME` not set)
    #[error("Couldn't extract an env found inside the setting file ({path:?}). {0}", path=PATHS.settings_file)]
    EnvExpansion(#[from] VarError),

    /// The settings.json file in *~/.config/ATA* could not be accessed
    ///
    /// It could either be absent, have had its name changed, or have gotten corrupted
    #[error("Couldn't access settings file ({path:?}). {0}", path=PATHS.settings_file)]
    SettingsFileAccessing(#[from] std::io::Error),

    /// The contents of settings.json were impossible to read
    ///
    /// This could be because the file is corrupted or contains invalid JSON
    #[error("Unable to read contents of settings file ({path:?}). {0}", path=PATHS.settings_file)]
    JsonReading(#[from] serde_json::Error),

    /// The provided setting name does not correspond to any known setting
    #[error("Unable to parse received setting name ({0}) into and actual setting")]
    InvalidSettingName(String),

    /// The provided value cannot be parsed into the type required by the target setting
    #[error(
        "Unable to parse received setting value ({0}) into a value acceptable for the given setting"
    )]
    InvalidSettingValue(String),
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

/// Expands shell variables and `~` in a path string and returns a [`PathBuf`]
///
/// # Arguments
/// * `value` - Raw path string, potentially containing `~` or `$VAR` references
///
/// # Returns
/// * [`Ok`] -> Expanded [`PathBuf`]
/// * [`Err`] -> [`SettingsInteractionError::EnvExpansion`] if a variable cannot be resolved
fn expand_path(value: &str) -> Result<PathBuf, SettingsInteractionError> {
    let expanded =
        full(value).map_err(|_| SettingsInteractionError::EnvExpansion(VarError::NotPresent))?;
    Ok(PathBuf::from(expanded.as_ref()))
}
