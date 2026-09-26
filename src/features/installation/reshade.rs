//! **reshade** recognizes ReShade presets and decides where their files go
//!
//! How presets are distributed (see `docs/RESHADE.md` for the full research):
//! * a preset is an `.ini` file listing the effects to enable (`Techniques=...`) and their
//!   settings (`[Effect.fx]` sections). It lives in the game folder, next to the game executable
//! * it may ship the custom effects it needs: `.fx`/`.fxh` shaders and textures, usually
//!   inside `reshade-shaders/Shaders` and `reshade-shaders/Textures`, which is where ReShade
//!   4.x looks for them by default
//! * archives often contain files ATA must not install: ReShade itself (`dxgi.dll`, ...),
//!   its configuration (`ReShade.ini`), logs, readmes and screenshots
//!
//! A preset is recognized by its content, not by its extension: any mod can ship an `.ini`,
//! but only presets contain a `Techniques=` or `TechniqueSorting=` key.
//!
//! Main function: [`analyse`]

use super::analysis::{ArchiveFile, ModFile};
use crate::data::mod_type::ModType;
use crate::features::game::RESHADE_BINARIES;

use std::fs;
use std::path::{Path, PathBuf};

/// Keys only found in ReShade preset files
const PRESET_KEYS: &[&str] = &["techniques", "techniquesorting"];
/// Files belonging to a ReShade installation rather than to a preset
const RESHADE_OWN_FILES: &[&str] = &["reshade.ini", "reshade.log", "reshade.log.old", "reshadepreset.ini.bak"];
/// Presets are small: bigger `.ini` files are not even read
const MAX_PRESET_SIZE: u64 = 1024 * 1024;
/// Folder (in the game root) where ReShade 4.x looks for effects and textures
pub(super) const RESHADE_SHADERS_DIR: &str = "reshade-shaders";

/// Recognizes a ReShade preset mod and lists the files to install
///
/// # Returns
/// * [`Some`] -> The archive contains at least one preset: its files and warnings about skipped files
/// * [`None`] -> The archive isn't a ReShade preset
pub(super) fn analyse(files: &[ArchiveFile]) -> Option<(Vec<ModFile>, Vec<String>)> {
    if !files.iter().any(|f| is_preset_file(&f.absolute)) {
        return None;
    }

    let mut mod_files = Vec::new();
    let mut warnings = Vec::new();

    for file in files {
        let name = file.file_name.to_ascii_lowercase();

        if RESHADE_BINARIES.contains(&name.as_str()) || RESHADE_OWN_FILES.contains(&name.as_str()) {
            warnings.push(format!(
                "Skipped '{}': it belongs to ReShade itself, which ATA doesn't install (tested version: 4.7.0)",
                file.relative.display()
            ));
        } else if is_preset_file(&file.absolute) {
            mod_files.push(ModFile::new(file, PathBuf::from(&file.file_name), ModType::ReshadePreset));
        } else if let Some(destination) = resource_destination(&file.relative, &file.extension) {
            mod_files.push(ModFile::new(file, destination, ModType::ReshadePreset));
        }
    }

    Some((mod_files, warnings))
}

/// Whether `path` is a ReShade preset (an `.ini` file with a `Techniques=`/`TechniqueSorting=` key)
pub(super) fn is_preset_file(path: &Path) -> bool {
    let name = path.file_name().unwrap_or_default().to_string_lossy().to_ascii_lowercase();
    let is_small = fs::metadata(path).is_ok_and(|m| m.len() <= MAX_PRESET_SIZE);

    if !name.ends_with(".ini") || RESHADE_OWN_FILES.contains(&name.as_str()) || !is_small {
        return false;
    }

    let Ok(bytes) = fs::read(path) else {
        return false;
    };

    String::from_utf8_lossy(&bytes).lines().any(|line| {
        line.split_once('=')
            .is_some_and(|(key, _)| PRESET_KEYS.contains(&key.trim().to_ascii_lowercase().as_str()))
    })
}

/// Where a shader/texture of a preset goes, relative to the game folder
///
/// * `.../reshade-shaders/<rest>` -> `reshade-shaders/<rest>`
/// * `.../Shaders/<rest>` or `.../Textures/<rest>` -> `reshade-shaders/Shaders|Textures/<rest>`
/// * a loose `.fx`/`.fxh` file -> `reshade-shaders/Shaders/<file>`
/// * anything else (readmes, screenshots...) -> [`None`]
fn resource_destination(relative: &Path, extension: &str) -> Option<PathBuf> {
    let components: Vec<String> = relative
        .iter()
        .map(|c| c.to_string_lossy().into_owned())
        .collect();
    let (folders, file_name) = components.split_at(components.len().saturating_sub(1));
    let file_name = file_name.first()?;

    let rebuild = |root: &str, rest: &[String]| -> PathBuf {
        let mut destination = PathBuf::from(RESHADE_SHADERS_DIR).join(root);
        destination.extend(rest);
        destination.push(file_name);
        destination
    };

    if let Some(i) = folders.iter().position(|f| f.eq_ignore_ascii_case(RESHADE_SHADERS_DIR)) {
        let mut destination = PathBuf::from(RESHADE_SHADERS_DIR);
        destination.extend(&folders[i + 1..]);
        destination.push(file_name);
        return Some(destination);
    }

    for root in ["Shaders", "Textures"] {
        if let Some(i) = folders.iter().position(|f| f.eq_ignore_ascii_case(root)) {
            return Some(rebuild(root, &folders[i + 1..]));
        }
    }

    matches!(extension, "fx" | "fxh").then(|| rebuild("Shaders", &[]))
}
