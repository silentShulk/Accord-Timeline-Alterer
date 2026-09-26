//! **mod_type** declares the kinds of mods ATA understands
//!
//! It knows how a single game file is recognized (extension + filename prefix)
//! and in which folder of the game it has to be installed.
//!
//! Main type: [`ModType`]

use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Mod types supported by ATA
///
/// Mod types not currently supported are not generic, but mod-specific (like NAIOM)
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Copy, Hash)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ModType {
    /// `DLL` mods are unique mods
    /// They contain **dll** files and whatever files sit next to them
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
    /// `Reshade presets` mods contain ReShade presets (**ini** files)
    /// and optionally the custom shaders/textures they need
    ReshadePreset,
}

impl ModType {
    /// Every mod type, in declaration order
    pub const ALL: &'static [ModType] = &[
        ModType::DLL,
        ModType::Textures,
        ModType::PlayerModels,
        ModType::WeaponModels,
        ModType::WorldModels,
        ModType::CutsceneReplacements,
        ModType::ReshadePreset,
    ];

    /// Recognizes a loose game file from its (lowercase) extension and filename
    ///
    /// DLL mods and ReShade presets are recognized per mod, not per file
    /// (see the installation analysis), so they are never returned here.
    ///
    /// # Returns
    /// * [`Some`] -> The type of mod the file belongs to
    /// * [`None`] -> The file isn't a game file ATA knows how to install
    pub fn of_game_file(extension: &str, file_name: &str) -> Option<ModType> {
        match (extension, file_prefix(file_name).as_str()) {
            ("dds", _) => Some(ModType::Textures),
            ("dtt" | "dat", "pl" | "mi") => Some(ModType::PlayerModels),
            ("dtt" | "dat", "wp") => Some(ModType::WeaponModels),
            ("dtt" | "dat", "bg") => Some(ModType::WorldModels),
            ("usm", _) => Some(ModType::CutsceneReplacements),
            _ => None,
        }
    }

    /// Returns the folder, relative to the game directory, where a game file of this type goes
    ///
    /// # Arguments
    /// * `mod_name` - Name of the mod (textures get their own folder)
    /// * `file_name` - Name of the file (its two-character prefix picks the model folder)
    ///
    /// # Returns
    /// * `wax/mods/<mod_name>` for textures
    /// * `data/pl`, `data/wp`, `data/bg` for models, `data/misctex` for `mi*` files
    /// * `data/movie` for cutscene replacements
    /// * the game root for DLL mods and ReShade presets
    pub fn destination_folder(&self, mod_name: &str, file_name: &str) -> PathBuf {
        match self {
            ModType::Textures => PathBuf::from("wax").join("mods").join(mod_name),
            ModType::PlayerModels | ModType::WeaponModels | ModType::WorldModels => {
                match file_prefix(file_name).as_str() {
                    "pl" => PathBuf::from("data").join("pl"),
                    "wp" => PathBuf::from("data").join("wp"),
                    "bg" => PathBuf::from("data").join("bg"),
                    _ => PathBuf::from("data").join("misctex"),
                }
            }
            ModType::CutsceneReplacements => PathBuf::from("data").join("movie"),
            ModType::DLL | ModType::ReshadePreset => PathBuf::new(),
        }
    }

    /// Picks the type that best describes a mod made of files of `types`
    ///
    /// A mod can legitimately mix types (e.g. a model shipping its textures): every file
    /// is still installed where its own type requires, this only chooses the label.
    ///
    /// # Returns
    /// * [`Some`] -> The most significant type (presets > DLL > models > textures > cutscenes)
    /// * [`None`] -> `types` is empty
    pub fn most_significant(types: &HashSet<ModType>) -> Option<ModType> {
        const PRIORITY: &[ModType] = &[
            ModType::ReshadePreset,
            ModType::DLL,
            ModType::PlayerModels,
            ModType::WeaponModels,
            ModType::WorldModels,
            ModType::Textures,
            ModType::CutsceneReplacements,
        ];

        PRIORITY.iter().copied().find(|t| types.contains(t))
    }

    /// Returns a short ID for the [`ModType`], used as part of a mod's [`crate::Mod::uid`]
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
    pub fn get_id(&self) -> &'static str {
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

    /// Human readable name of the type
    pub fn label(&self) -> &'static str {
        match self {
            ModType::DLL => "Unique mod",
            ModType::Textures => "Textures",
            ModType::PlayerModels => "Player Models",
            ModType::WeaponModels => "Weapon Models",
            ModType::WorldModels => "World Models",
            ModType::CutsceneReplacements => "Cutscene Replacements",
            ModType::ReshadePreset => "ReShade Preset",
        }
    }
}

impl fmt::Display for ModType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// First two characters of a filename, lowercased (`"PL0000.dat"` -> `"pl"`)
fn file_prefix(file_name: &str) -> String {
    file_name.chars().take(2).collect::<String>().to_ascii_lowercase()
}
