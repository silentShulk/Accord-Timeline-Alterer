//! **data** is a module that declares types and functions
//! for interacting with saved mod data
//!
//! In the case of ATA the saved data is stored inside a
//! "data.json" file
//!
//! This includes:
//! * **loading**: Reading the data file into a [`Data`] struct
//! * **saving**: Writing the current in-memory state back to the data file
//! * **querying**: Looking up mods by name or checking for name conflicts
//! * **mutating**: Adding, removing, and toggling the enabled state of mods
//!
//! Main type: [`Data`]

use crate::paths::PATHS;
use crate::utils::files::{expand_path, FilesInteractionError};
use crate::utils::json::{JsonParsingError, load_json, save_json};

use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use std::fmt::{self};

use thiserror::Error;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use strum::{EnumIter, IntoEnumIterator};



/// Holds the runtime state of ATA: the full list of installed mods
#[derive(Serialize, Deserialize)]
pub struct Data {
    /// List of all mods currently tracked by ATA
    pub mods: Vec<Mod>,
}

impl Data {
    /// Creates a [`Data`] instance from the data file
    ///
    /// Also expands any shell variables or `~` present in stored file paths.
    ///
    /// # Returns
    /// * [`Ok`] -> A [`Data`] instance populated from the data file
    /// * [`Err`] -> The type of error that occurred
    ///
    /// # Errors
    /// * [`DataInteractionError::Json`] if the data file cannot be opened, read, or parsed as JSON
    /// * [`DataInteractionError::FilesInteraction`] if a stored file path cannot be shell-expanded
    pub fn load_data() -> Result<Self, DataInteractionError> {
        let mut contents : Data= load_json(&PATHS.data_file)?;

        contents.mods.iter_mut().try_for_each(|m| -> Result<(), FilesInteractionError> {
            m.files = m.files.iter()
                .map(|f| expand_path(&f.to_string_lossy()))
                .collect::<Result<_, _>>()?;
            Ok(())
        })?;

        Ok(contents)
    }

    /// Checks whether a mod with the given name is already tracked
    ///
    /// # Arguments
    /// * `name` - The candidate mod name to check
    ///
    /// # Returns
    /// * `true` if a mod with `name` already exists
    /// * `false` if the name is free to use
    pub fn name_exists(&self, name: &str) -> bool {
        self.mods.iter().any(|m| m.name == name)
    }

    /// Removes conflicting files from currently tracked mods by discarding them from the older mod's file list
    ///
    /// # Arguments
    /// * `conflicts_list` - A map where the key is the conflicting file path and the value is the name of the existing mod owning it
    ///
    /// # Panics
    /// Panics if `conflicts_list` references a mod name not present in [`Data::mods`].
    /// Callers must build `conflicts_list` from this same [`Data`] instance (see [`crate::installation::check_for_conflicts`])
    /// so the invariant always holds in practice.
    pub fn remove_conflicts(&mut self, conflicts_list: &HashMap<PathBuf, String>) {
        for conflict in conflicts_list {
            let conflicting_mod_idx = self.get_mod_by_name(conflict.1.as_ref()).unwrap().0;
            let conflict_filename = conflict.0.file_name();
            self.mods[conflicting_mod_idx]
                .files
                .retain(|f| f.file_name() != conflict_filename);
        }
    }

    /// Appends `new_mod` to the in-memory list and writes the data file
    ///
    /// # Arguments
    /// * `new_mod` - The [`Mod`] to add
    ///
    /// # Returns
    /// * [`Ok`] -> `()` on success
    /// * [`Err`] -> The type of error that occurred
    ///
    /// # Errors
    /// * [`DataInteractionError::Json`] if the data file cannot be written or serialized
    pub fn save_new_mod(&mut self, new_mod: &Mod) -> Result<(), DataInteractionError> {
        self.mods.push(new_mod.clone());
        Ok(save_json(&PATHS.data_file, &self)?)
    }

    /// Removes the mod at `index_to_remove` from the in-memory list and writes the data file
    ///
    /// # Arguments
    /// * `index_to_remove` - Index into [`Data::mods`] of the mod to remove
    ///
    /// # Returns
    /// * [`Ok`] -> `()` on success
    /// * [`Err`] -> The type of error that occurred
    ///
    /// # Errors
    /// * [`DataInteractionError::Json`] if the data file cannot be written or serialized
    pub fn remove_mod(&mut self, index_to_remove: usize) -> Result<(), DataInteractionError> {
        self.mods.remove(index_to_remove);
        Ok(save_json(&PATHS.data_file, &self)?)
    }

    /// Toggles the enabled flag of the mod at `index`, replaces its file list, then writes the data file
    ///
    /// # Arguments
    /// * `index` - Index into [`Data::mods`] of the mod to update
    /// * `new_files` - Updated list of file paths to store (reflecting the new enabled/disabled location)
    ///
    /// # Returns
    /// * [`Ok`] -> `()` on success
    /// * [`Err`] -> The type of error that occurred
    ///
    /// # Errors
    /// * [`DataInteractionError::Json`] if the data file cannot be written or serialized
    pub fn switch_mod_state(
        &mut self,
        index: usize,
        new_files: Vec<PathBuf>,
    ) -> Result<(), DataInteractionError> {
        self.mods[index].files = new_files;
        self.mods[index].enabled = !self.mods[index].enabled;
        Ok(save_json(&PATHS.data_file, &self)?)
    }

    /// Finds the mod whose name matches `name`
    ///
    /// # Arguments
    /// * `name` - The name to search for
    ///
    /// # Returns
    /// * [`Ok`]`((usize, Mod))` — index in [`Data::mods`] and a clone of the matching mod
    /// * [`Err`] -> [`DataInteractionError::ModNotFound`] if no mod with that name exists
    pub fn get_mod_by_name(&self, name: &str) -> Result<(usize, Mod), DataInteractionError> {
        self.mods
            .iter()
            .enumerate()
            .find(|(_, m)| m.name == name)
            .map(|(i, m)| (i, m.clone()))
            .ok_or_else(|| DataInteractionError::ModNotFound(name.to_string()))
    }
}

/// Errors that could occur during interactions with the saved data
#[derive(Error, Debug)]
pub enum DataInteractionError {
    /// The file extension does not correspond to any known mod type recognized by ATA
    #[error("'{0}' is not an extension of a know mod type")]
    InvalidModTypeExtension(String),

    /// The mod folder contains files that prevent inferring a clear mod type
    #[error("Couldn't not infer mod type from mod files")]
    UnclearModType,

    /// Reading or writing the data file as JSON failed
    #[error("Could'not parse Json for saving/loading data. {0}")]
    Json(#[from] JsonParsingError),

    /// A path component of a stored file could not be extracted or expanded
    #[error("An error occurred while interacting with files. {0}")]
    FilesInteraction(#[from] FilesInteractionError),

    /// No mod with the given name was found in the data file
    #[error("No installed mod has the name {0}")]
    ModNotFound(String),
}

/// Everything ATA needs to track about an installed mod
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub struct Mod {
    /// Name of the mod given by the user
    pub name: String,
    /// Individual files belonging to the mod (full paths, not the containing folder)
    pub files: Vec<PathBuf>,
    /// Whether the mod is currently active in the game folder
    pub enabled: bool,
    /// Categorises what kind of assets the mod replaces
    pub mod_type: ModType,
    /// UTC timestamp of when the mod was installed
    pub install_date: DateTime<Utc>,
    /// Unique identifier derived from the mod's name, type, and install date
    pub uid: String,
}

impl Mod {
    /// Creates a new [`Mod`] and generates its [`Mod::uid`] automatically
    ///
    /// # Arguments
    /// * `name` - Name of the mod given by the user
    /// * `files` - Individual files belonging to the mod (full paths, not the containing folder)
    /// * `enabled` - Whether the mod is currently active in the game folder
    /// * `mod_type` - Categorises what kind of assets the mod replaces
    /// * `install_date` - UTC timestamp of when the mod was installed
    ///
    /// # Returns
    /// * A fully populated [`Mod`] instance with a generated [`Mod::uid`]
    pub fn new(
        name: String,
        files: Vec<PathBuf>,
        enabled: bool,
        mod_type: ModType,
        install_date: DateTime<Utc>,
    ) -> Self {
        Self {
            uid: Self::get_uid(&name, &mod_type, &install_date),
            name,
            files,
            enabled,
            mod_type,
            install_date,
        }
    }

    /// Builds a unique identifier string from the mod's name, type, and install date
    ///
    /// The UID is formed by concatenating the first four characters of the name,
    /// the type's short ID (see [`ModType::get_id`]), and the install date formatted
    /// as `dd/mm/yyyy|HH:MM`.
    ///
    /// # Arguments
    /// * `mod_name` - Name of the mod given by the user
    /// * `mod_type` - Type of the mod
    /// * `install_date` - UTC timestamp of when the mod was installed
    ///
    /// # Returns
    /// * A [`String`] containing the generated UID
    fn get_uid(mod_name: &str, mod_type: &ModType, install_date: &DateTime<Utc>) -> String {
        let name: String = mod_name.chars().take(4).collect();
        let m_type = mod_type.get_id();
        let date = install_date.format("%d/%m/%Y|%H:%M").to_string();

        format!("{}{}{}", name, m_type, date)
    }
}

/// Mod types supported by ATA
///
/// Mod types not currently supported are not generic, but mod-specific (like NAIOM)
#[derive(
    Serialize, Deserialize, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Copy, Hash, EnumIter,
)]
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
    /// `Cutscene replacements` mods contain replacements for the game's cutscenes
    /// They contain **usm** files
    CutsceneReplacements,
    /// `Reshade presets` mods contain shader presets
    /// They contain **ini** files and other files
    ReshadePreset,
}

impl ModType {
    /// Returns the relative subfolder inside the game directory where this mod type's files live
    ///
    /// # Arguments
    /// * `mod_name` - Name of the mod
    /// * `prefix` - Two-character prefix extracted from the filename (e.g., `"pl"`, `"wp"`, `"bg"`)
    ///
    /// # Returns
    /// * `wax/mods/<mod_name>` for textures
    /// * `data/pl/` or `data/misctex/` for player models
    /// * `data/wp/` or `data/misctex/` for weapon models
    /// * `data/bg/` or `data/misctex/` for world models
    /// * `data/movie/` for cutscene replacements
    /// * empty path for reshade presets & DLL mods (game root)
    pub fn get_corresponding_folder(&self, mod_name: &String, prefix: &str) -> PathBuf {
        match self {
            ModType::DLL => PathBuf::new(),
            ModType::Textures => PathBuf::from("wax").join("mods").join(mod_name),
            ModType::PlayerModels => {
                if prefix == "pl" {
                    PathBuf::from("data").join("pl")
                } else {
                    PathBuf::from("data").join("misctex")
                }
            }
            ModType::WeaponModels => {
                if prefix == "wp" {
                    PathBuf::from("data").join("wp")
                } else {
                    PathBuf::from("data").join("misctex")
                }
            }
            ModType::WorldModels => {
                if prefix == "bg" {
                    PathBuf::from("data").join("bg")
                } else {
                    PathBuf::from("data").join("misctex")
                }
            }
            ModType::CutsceneReplacements => PathBuf::from("data").join("movie"),
            ModType::ReshadePreset => PathBuf::new(),
        }
    }

    /// Returns a short ID for the [`ModType`], used as part of a mod's [`Mod::uid`]
    ///
    /// # Returns
    /// A string slice containing the ID:
    /// * `DLL` -> `"Dll"`
    /// * `Textures` -> `"Te"`
    /// * `PlayerModels` -> `"PlMo"`
    /// * `WeaponModels` -> `"WeMo"`
    /// * `WorldModels` -> `"WoMo"`
    /// * `CutsceneReplacements` -> `"CuRe"`
    /// * `ReshadePreset` -> `"RePr"`
    fn get_id(&self) -> &str {
        match self {
            ModType::DLL => "Dll",
            ModType::Textures => "Te",
            ModType::PlayerModels => "PlMo",
            ModType::WeaponModels => "WeMo",
            ModType::WorldModels => "WoMo",
            ModType::CutsceneReplacements => "CuRe",
            ModType::ReshadePreset => "RePr",
        }
    }

    /// Returns a set of all file extensions recognized by ATA across all supported mod types
    ///
    /// # Returns
    /// * A [`HashSet`] of string slices containing extensions (`"dll"`, `"dds"`, `"dtt"`, `"dat"`, `"usm"`, `"ini"`)
    pub fn all_extensions() -> HashSet<&'static str> {
        ModType::iter()
            .flat_map(|t| match t {
                ModType::DLL => ["dll"].as_slice(),
                ModType::Textures => ["dds"].as_slice(),
                ModType::PlayerModels | ModType::WeaponModels | ModType::WorldModels => {
                    ["dtt", "dat"].as_slice()
                }
                ModType::CutsceneReplacements => ["usm"].as_slice(),
                ModType::ReshadePreset => ["ini"].as_slice(),
            })
            .copied()
            .collect()
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

impl TryFrom<(&str, &str)> for ModType {
    type Error = DataInteractionError;

    fn try_from(file_description: (&str, &str)) -> Result<Self, DataInteractionError> {
        match file_description {
            ("dll", _) => Ok(ModType::DLL),
            ("dds", _) => Ok(ModType::Textures),
            ("dtt" | "dat", "pl") => Ok(ModType::PlayerModels),
            ("dtt" | "dat", "mi") => Ok(ModType::PlayerModels),
            ("dtt" | "dat", "wp") => Ok(ModType::WeaponModels),
            ("dtt" | "dat", "bg") => Ok(ModType::WorldModels),
            ("usm", _) => Ok(ModType::CutsceneReplacements),
            ("ini", _) => Ok(ModType::ReshadePreset),
            (ext, _) => Err(DataInteractionError::InvalidModTypeExtension(
                ext.to_string(),
            )),
        }
    }
}

impl TryFrom<HashSet<ModType>> for ModType {
    type Error = DataInteractionError;

    fn try_from(mod_types: HashSet<ModType>) -> Result<Self, DataInteractionError> {
        if mod_types.len() == 1 {
            Ok(mod_types.into_iter().next().unwrap())
        } else if mod_types.contains(&ModType::PlayerModels) {
            Ok(ModType::PlayerModels)
        } else if mod_types.contains(&ModType::WeaponModels) {
            Ok(ModType::WeaponModels)
        } else if mod_types.contains(&ModType::WorldModels) {
            Ok(ModType::WorldModels)
        } else {
            Err(DataInteractionError::UnclearModType)
        }
    }
}