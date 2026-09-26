//! **settings** is a module that declares types and functions
//! for interacting with user settings
//!
//! In the case of ATA the user's settings are saved inside a
//! "settings.json" file in the OS config directory (see [`crate::PATHS`])
//!
//! This includes:
//! * **loading**: Reading the settings file into a [`Settings`] struct
//! * **saving**: Writing the current in-memory settings back to the settings file
//! * **updating**: Parsing and applying a single setting change by name and value
//!
//! Every field has a default, so a settings file missing some keys (e.g. written by an
//! older version of ATA) is still loaded correctly.
//!
//! Main type: [`Settings`]

use crate::data::paths::PATHS;
use crate::utils::files::{FilesInteractionError, expand_path};
use crate::utils::json::{JsonParsingError, load_json, save_json};

use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Name of the GUI shipped with ATA, used as the default [`Settings::style`]
pub const DEFAULT_STYLE: &str = "ShellUI";

/// All user-configurable settings for ATA
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Settings {
    /// Name of the GUI (see the Launcher) the user last chose
    pub style: String,
    /// Color palette applied to the UI
    pub palette: Palette,
    /// Order in which mods are shown in the list view
    pub sorting_order: SortingOrder,

    /// How to handle file conflicts when installing a mod
    pub files_conflict_resolution: ConflictResolution,
    /// Whether to keep the extracted temporary folder after an installation
    pub keep_extracted_folders: bool,
    /// Folder in which archives are extracted during installation.
    /// Empty means the OS temporary folder
    pub extracted_folders_location: PathBuf,
    /// Absolute path to the game's installation folder
    pub game_path: PathBuf,

    /// Text shown on Discord Rich Presence by GUIs supporting it; empty string means disabled
    pub discord_rich_presence: String,

    /// File these settings were loaded from, and will be saved to
    #[serde(skip)]
    source: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            style: DEFAULT_STYLE.to_string(),
            palette: Palette::default(),
            sorting_order: SortingOrder::default(),
            files_conflict_resolution: ConflictResolution::default(),
            keep_extracted_folders: false,
            extracted_folders_location: PathBuf::new(),
            game_path: PathBuf::new(),
            discord_rich_presence: String::new(),
            source: None,
        }
    }
}

impl Settings {
    /// Creates a [`Settings`] instance from the settings file (see [`crate::Paths::settings_file`])
    ///
    /// # Errors
    /// See [`Settings::load_from`]
    pub fn load_settings() -> Result<Self, SettingsInteractionError> {
        Self::load_from(&PATHS.settings_file)
    }

    /// Creates a [`Settings`] instance from the given settings file
    ///
    /// Also expands any shell variables or `~` present in `game_path` and `extracted_folders_location`.
    ///
    /// # Returns
    /// * [`Ok`] -> A [`Settings`] instance populated from the settings file
    /// * [`Err`] -> The type of error that occurred
    ///
    /// # Errors
    /// * [`SettingsInteractionError::Json`] if the settings file cannot be opened, read, or parsed
    /// * [`SettingsInteractionError::FilesInteraction`] if a shell variable in a path cannot be resolved
    pub fn load_from(settings_file: &Path) -> Result<Self, SettingsInteractionError> {
        let mut settings: Settings = load_json(settings_file)?;

        for path in [&mut settings.game_path, &mut settings.extracted_folders_location] {
            *path = expand_path(&path.to_string_lossy())?;
        }
        settings.source = Some(settings_file.to_path_buf());

        Ok(settings)
    }

    /// The file these settings are saved to
    pub fn file(&self) -> &Path {
        self.source.as_deref().unwrap_or(&PATHS.settings_file)
    }

    /// Writes the settings to [`Settings::file`]
    ///
    /// # Errors
    /// * [`SettingsInteractionError::Json`] if the settings file cannot be written
    pub fn save(&self) -> Result<(), SettingsInteractionError> {
        Ok(save_json(self.file(), self)?)
    }

    /// Writes the settings to `settings_file`, which becomes their new [`Settings::file`]
    ///
    /// # Errors
    /// * [`SettingsInteractionError::Json`] if the settings file cannot be written
    pub fn save_to(&mut self, settings_file: &Path) -> Result<(), SettingsInteractionError> {
        self.source = Some(settings_file.to_path_buf());
        self.save()
    }

    /// Folder in which mod archives get extracted ([`Settings::extracted_folders_location`],
    /// or `<OS temp folder>/ATA` when that is empty)
    pub fn extraction_dir(&self) -> PathBuf {
        if self.extracted_folders_location.as_os_str().is_empty() {
            std::env::temp_dir().join("ATA")
        } else {
            self.extracted_folders_location.clone()
        }
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
    pub fn update_setting(&mut self, setting: &str, value: &str) -> Result<Settings, SettingsInteractionError> {
        let mut updated = self.clone();

        match setting {
            "style" => updated.style = value.to_string(),
            "palette" => updated.palette = value.parse()?,
            "sortingOrder" => updated.sorting_order = value.parse()?,
            "filesConflictResolution" => updated.files_conflict_resolution = value.parse()?,
            "keepExtractedFolders" => {
                updated.keep_extracted_folders = value
                    .trim()
                    .parse()
                    .map_err(|_| SettingsInteractionError::InvalidSettingValue(value.to_string()))?
            }
            "extractedFoldersLocation" => updated.extracted_folders_location = expand_path(value.trim())?,
            "gamePath" => updated.game_path = expand_path(value.trim())?,
            "discordRichPresence" => updated.discord_rich_presence = value.to_string(),
            _ => return Err(SettingsInteractionError::InvalidSettingName(setting.to_string())),
        };

        // Only touch the in-memory settings once the file has really been written
        updated.save()?;
        *self = updated;

        Ok(self.clone())
    }
}

/// Errors that could occur during interactions with the settings file
#[derive(Error, Debug)]
pub enum SettingsInteractionError {
    /// The provided setting name does not correspond to any known setting
    #[error("'{0}' is not a setting")]
    InvalidSettingName(String),

    /// The provided value cannot be parsed into the type required by the target setting
    #[error("'{0}' is not a valid value for the given setting")]
    InvalidSettingValue(String),

    /// Reading or writing the settings file as JSON failed
    #[error("Couldn't load/save settings. {0}")]
    Json(#[from] JsonParsingError),

    /// A path component of a setting value could not be extracted or expanded
    #[error("An error occurred while interacting with files. {0}")]
    FilesInteraction(#[from] FilesInteractionError),
}

/// Parses a setting value by reusing the serde representation of `T`
/// (so accepted values, aliases included, can never drift from what settings.json contains)
fn parse_setting_value<T: DeserializeOwned>(value: &str) -> Result<T, SettingsInteractionError> {
    serde_json::from_value(serde_json::Value::String(value.trim().to_string()))
        .map_err(|_| SettingsInteractionError::InvalidSettingValue(value.to_string()))
}

/// Implements [`FromStr`] (through [`parse_setting_value`]) and exposes every variant in `ALL`
macro_rules! setting_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        impl $name {
            /// Every possible value, in declaration order
            pub const ALL: &'static [$name] = &[$($name::$variant),+];
        }

        impl FromStr for $name {
            type Err = SettingsInteractionError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                parse_setting_value(s)
            }
        }
    };
}

/// The color palette applied to the UI
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Palette {
    /// Default palette, inspired by NieR: Automata
    #[default]
    Automata,
    /// Alternative palette, inspired by NieR: Replicant
    Replicant,
}
setting_enum!(Palette { Automata, Replicant });

/// The order in which installed mods are displayed in the list view
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum SortingOrder {
    /// Group mods by their [`crate::ModType`] (default)
    #[default]
    ModType,
    /// Sort mods from newest to oldest install date
    #[serde(alias = "Install Date")]
    InstallDate,
    /// Sort enabled mods before disabled ones
    #[serde(alias = "Enable Status")]
    EnableStatus,
    /// Sort mods alphabetically by name (case insensitive)
    Alphabetical,
    /// Sort mods from largest to smallest total file size
    Size,
}
setting_enum!(SortingOrder { ModType, InstallDate, EnableStatus, Alphabetical, Size });

/// What ATA does when a mod file would overwrite a file of an already installed mod
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ConflictResolution {
    /// Refuse the installation and report the conflicts, unless an overwrite is forced (default)
    ///
    /// `"Ask"` is accepted as an alias: older Launchers wrote it in the default settings
    #[default]
    #[serde(alias = "Ask")]
    Warn,
    /// Silently overwrite the existing file with the mod's version
    Overwrite,
}
setting_enum!(ConflictResolution { Warn, Overwrite });
