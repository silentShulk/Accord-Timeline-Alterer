//! **mods** is a module that declares types and functions
//! for interacting with saved mod data
//!
//! In the case of ATA the saved data is stored inside a
//! "data.json" file (see [`crate::Paths::data_file`])
//!
//! This includes:
//! * **loading**: Reading the data file into a [`Mods`] struct
//! * **saving**: Writing the current in-memory state back to the data file
//! * **querying**: Looking up mods by name or by the files they own
//!
//! Main types: [`Mods`], [`Mod`]

use crate::data::mod_type::ModType;
use crate::data::paths::PATHS;
use crate::utils::files::{DISABLED_DIR_NAME, FilesInteractionError, expand_path};
use crate::utils::json::{JsonParsingError, load_json, save_json};

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Holds the runtime state of ATA: the full list of installed mods
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Mods {
    /// List of all mods currently tracked by ATA, in installation order
    pub mods: Vec<Mod>,

    /// File the data was loaded from, and will be saved to
    #[serde(skip)]
    source: Option<PathBuf>,
}

impl Mods {
    /// Creates a [`Mods`] instance from the data file (see [`crate::Paths::data_file`])
    ///
    /// # Errors
    /// See [`Mods::load_from`]
    pub fn load_data() -> Result<Self, DataInteractionError> {
        Self::load_from(&PATHS.data_file)
    }

    /// Creates a [`Mods`] instance from the given data file
    ///
    /// A missing data file simply means that no mod was installed yet: it is
    /// treated as an empty list and will be created on the first save.
    /// Also expands any shell variables or `~` present in stored file paths.
    ///
    /// # Returns
    /// * [`Ok`] -> A [`Mods`] instance populated from the data file
    /// * [`Err`] -> The type of error that occurred
    ///
    /// # Errors
    /// * [`DataInteractionError::Json`] if the data file cannot be read, or parsed as JSON
    /// * [`DataInteractionError::FilesInteraction`] if a stored file path cannot be shell-expanded
    pub fn load_from(data_file: &Path) -> Result<Self, DataInteractionError> {
        let mut data: Mods = match load_json(data_file) {
            Ok(data) => data,
            Err(err) if err.is_not_found() => Mods::default(),
            Err(err) => return Err(err.into()),
        };

        for m in &mut data.mods {
            for f in &mut m.files {
                *f = expand_path(&f.to_string_lossy())?;
            }
        }
        data.source = Some(data_file.to_path_buf());

        Ok(data)
    }

    /// The file this data is saved to
    pub fn file(&self) -> &Path {
        self.source.as_deref().unwrap_or(&PATHS.data_file)
    }

    /// Writes the current state to [`Mods::file`]
    ///
    /// # Errors
    /// * [`DataInteractionError::Json`] if the data file cannot be written or serialized
    pub fn save(&self) -> Result<(), DataInteractionError> {
        Ok(save_json(self.file(), self)?)
    }

    /// Checks whether a mod with the given name is already tracked
    ///
    /// Names are compared case-insensitively: texture mods are installed in a folder
    /// named after the mod, and `Foo` and `foo` are the same folder on Windows.
    pub fn name_exists(&self, name: &str) -> bool {
        self.mods.iter().any(|m| m.name.eq_ignore_ascii_case(name.trim()))
    }

    /// Finds the index of the mod whose name matches `name`
    ///
    /// An exact match is preferred; otherwise a case-insensitive match is accepted.
    ///
    /// # Errors
    /// * [`DataInteractionError::ModNotFound`] if no mod with that name exists
    pub fn index_of(&self, name: &str) -> Result<usize, DataInteractionError> {
        self.mods
            .iter()
            .position(|m| m.name == name)
            .or_else(|| self.mods.iter().position(|m| m.name.eq_ignore_ascii_case(name)))
            .ok_or_else(|| DataInteractionError::ModNotFound(name.to_string()))
    }

    /// Returns the mod whose name matches `name` (see [`Mods::index_of`])
    ///
    /// # Errors
    /// * [`DataInteractionError::ModNotFound`] if no mod with that name exists
    pub fn get(&self, name: &str) -> Result<&Mod, DataInteractionError> {
        Ok(&self.mods[self.index_of(name)?])
    }

    /// Finds which mod, among those other than `excluded`, owns the file whose
    /// enabled location is `active_path`
    ///
    /// # Returns
    /// * [`Some`] -> Index of the owning mod
    /// * [`None`] -> No (other) mod owns that file
    pub fn owner_of(&self, active_path: &Path, excluded: Option<usize>) -> Option<usize> {
        self.mods
            .iter()
            .enumerate()
            .filter(|(i, _)| Some(*i) != excluded)
            .find(|(_, m)| m.files.iter().any(|f| same_path(&Mod::active_path(f), active_path)))
            .map(|(i, _)| i)
    }
}

/// Errors that could occur during interactions with the saved data
#[derive(Error, Debug)]
pub enum DataInteractionError {
    /// Reading or writing the data file as JSON failed
    #[error("Couldn't load/save installed mods data. {0}")]
    Json(#[from] JsonParsingError),

    /// A path component of a stored file could not be extracted or expanded
    #[error("An error occurred while interacting with files. {0}")]
    FilesInteraction(#[from] FilesInteractionError),

    /// No mod with the given name was found in the data file
    #[error("No installed mod has the name '{0}'")]
    ModNotFound(String),
}

/// Everything ATA needs to track about an installed mod
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Mod {
    /// Name of the mod given by the user
    pub name: String,
    /// Individual files belonging to the mod (full paths, not the containing folder).
    /// Disabled files live in a `.disabled/` folder next to their enabled location
    pub files: Vec<PathBuf>,
    /// Whether the mod is currently active in the game folder
    pub enabled: bool,
    /// Categorises what kind of assets the mod replaces
    pub mod_type: ModType,
    /// UTC timestamp of when the mod was installed
    pub install_date: DateTime<Utc>,
    /// Identifier derived from the mod's name, type, and install date
    pub uid: String,
    /// Original game files the mod replaced, saved so they can be restored
    #[serde(default)]
    pub backups: Vec<Backup>,
}

/// An original file of the game that was set aside because a mod replaced it
///
/// While the mod is enabled the original lives at `backup`;
/// while it is disabled the original is put back at `original`.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Backup {
    /// Where the original file belongs (= enabled location of the mod file replacing it)
    pub original: PathBuf,
    /// Where the original file is kept while the mod is enabled
    pub backup: PathBuf,
}

impl Mod {
    /// Creates a new, enabled, [`Mod`] installed now and generates its [`Mod::uid`]
    ///
    /// # Arguments
    /// * `name` - Name of the mod given by the user
    /// * `files` - Individual files belonging to the mod (full paths, not the containing folder)
    /// * `mod_type` - Categorises what kind of assets the mod replaces
    /// * `backups` - Original game files replaced by the mod
    pub fn new(name: String, files: Vec<PathBuf>, mod_type: ModType, backups: Vec<Backup>) -> Self {
        let install_date = Utc::now();

        Self {
            uid: Self::get_uid(&name, &mod_type, &install_date),
            name,
            files,
            enabled: true,
            mod_type,
            install_date,
            backups,
        }
    }

    /// Builds an identifier string from the mod's name, type, and install date
    ///
    /// The UID is formed by concatenating the first four characters of the name,
    /// the type's short ID (see [`ModType::get_id`]), and the install date formatted
    /// as `dd/mm/yyyy|HH:MM:SS`.
    fn get_uid(mod_name: &str, mod_type: &ModType, install_date: &DateTime<Utc>) -> String {
        let name: String = mod_name.chars().take(4).collect();
        let date = install_date.format("%d/%m/%Y|%H:%M:%S");

        format!("{}{}{}", name, mod_type.get_id(), date)
    }

    /// Location a mod file has while its mod is enabled
    ///
    /// `game/data/pl/.disabled/pl0000.dat` -> `game/data/pl/pl0000.dat`
    pub fn active_path(file: &Path) -> PathBuf {
        match (file.parent(), file.file_name()) {
            (Some(parent), Some(name)) if parent.file_name() == Some(OsStr::new(DISABLED_DIR_NAME)) => {
                parent.parent().unwrap_or(parent).join(name)
            }
            _ => file.to_path_buf(),
        }
    }

    /// Location a mod file has while its mod is disabled
    ///
    /// `game/data/pl/pl0000.dat` -> `game/data/pl/.disabled/pl0000.dat`
    pub fn disabled_path(file: &Path) -> PathBuf {
        let active = Self::active_path(file);
        match (active.parent(), active.file_name()) {
            (Some(parent), Some(name)) => parent.join(DISABLED_DIR_NAME).join(name),
            _ => active,
        }
    }

    /// Whether a mod file stays in place when its mod is disabled
    ///
    /// ReShade shaders and textures (anything inside `reshade-shaders/`) are only used through
    /// presets and are often shared between presets, so disabling a preset only moves its `.ini`.
    pub fn is_static_file(file: &Path) -> bool {
        file.components()
            .any(|c| c.as_os_str().to_string_lossy().eq_ignore_ascii_case("reshade-shaders"))
    }

    /// Returns the backup of the original file that `active_path` replaced, if any
    pub fn backup_for(&self, active_path: &Path) -> Option<&Backup> {
        self.backups.iter().find(|b| same_path(&b.original, active_path))
    }

    /// Removes and returns the backup of the original file that `active_path` replaced, if any
    pub fn take_backup_for(&mut self, active_path: &Path) -> Option<Backup> {
        let index = self.backups.iter().position(|b| same_path(&b.original, active_path))?;
        Some(self.backups.remove(index))
    }

    /// Total size on disk of the mod's files, in bytes (missing files count as 0)
    pub fn total_size(&self) -> u64 {
        self.files
            .iter()
            .filter_map(|f| f.metadata().ok())
            .map(|m| m.len())
            .sum()
    }
}

/// Compares two paths the way the current OS file system does
/// (case-insensitively and ignoring the separator kind on Windows)
pub fn same_path(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    {
        let normalize = |p: &Path| p.to_string_lossy().replace('/', "\\").to_lowercase();
        normalize(a) == normalize(b)
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}
