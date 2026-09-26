//! **analysis** looks at the files of an extracted mod to understand what it is
//! and where every file has to be installed
//!
//! A mod is recognized in this order:
//! 1. **ReShade preset**: it contains a preset `.ini` (see [`super::reshade`])
//! 2. **DLL mod**: it contains a `.dll`. Everything inside the folder of the least nested
//!    `.dll` is installed in the game root keeping its structure (so both two-file mods and
//!    mods with their own folders work), except documentation files
//! 3. **Game files**: every `.dds`, `.dtt`/`.dat` (`pl`, `wp`, `bg`, `mi` prefixes) and `.usm`
//!    file goes to its own game folder (see [`ModType::destination_folder`]); anything else is ignored
//!
//! Main function: [`analyse`]

use super::archive::EXTRACTION_MARKER;
use super::{InstallationError, reshade};
use crate::data::mod_type::ModType;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// Names (lowercase prefixes) of documentation files never installed from DLL mods
const DOCUMENTATION_PREFIXES: &[&str] = &["readme", "read me", "license", "licence", "changelog", "credits"];
/// Extensions of documentation files never installed from DLL mods
const DOCUMENTATION_EXTENSIONS: &[&str] = &["md", "pdf", "url", "htm", "html", "lnk"];

/// A file found inside the mod folder
pub(super) struct ArchiveFile {
    /// Full path of the file
    pub absolute: PathBuf,
    /// Path of the file relative to the mod folder
    pub relative: PathBuf,
    /// Name of the file
    pub file_name: String,
    /// Lowercase extension of the file (empty if none)
    pub extension: String,
}

/// A file that has to be installed
#[derive(Debug, Clone)]
pub(super) struct ModFile {
    /// Where the file currently is (inside the mod folder)
    pub source: PathBuf,
    /// Where the file goes, relative to the game folder
    pub destination: PathBuf,
    /// Type of mod the file belongs to
    pub mod_type: ModType,
}

impl ModFile {
    pub fn new(file: &ArchiveFile, destination: PathBuf, mod_type: ModType) -> Self {
        Self {
            source: file.absolute.clone(),
            destination,
            mod_type,
        }
    }
}

/// What the analysis found out about a mod
pub(super) struct ModAnalysis {
    /// Files to install
    pub files: Vec<ModFile>,
    /// Type describing the mod as a whole
    pub mod_type: ModType,
    /// Things the user should know (e.g. skipped files)
    pub warnings: Vec<String>,
}

/// Inspects the files of the mod folder to infer the mod type and where each file goes
///
/// # Arguments
/// * `mod_folder` - Path to the extracted mod folder
/// * `mod_name` - Name given to the mod (texture mods get a folder named after it)
///
/// # Errors
/// * [`InstallationError::EntryReading`] if reading directory entries fails
/// * [`InstallationError::ModlessFolder`] if no file of the folder could be recognized
pub(super) fn analyse(mod_folder: &Path, mod_name: &str) -> Result<ModAnalysis, InstallationError> {
    let (files, mut warnings) = collect_files(mod_folder)?;

    let mod_files = if let Some((preset_files, preset_warnings)) = reshade::analyse(&files) {
        warnings.extend(preset_warnings);
        preset_files
    } else if let Some(dll_files) = analyse_dll_mod(&files, mod_name) {
        dll_files
    } else {
        files.iter().filter_map(|f| game_file(f, mod_name)).collect()
    };

    let types: HashSet<ModType> = mod_files.iter().map(|f| f.mod_type).collect();
    let mod_type = ModType::most_significant(&types)
        .ok_or_else(|| InstallationError::ModlessFolder(mod_folder.to_path_buf()))?;

    Ok(ModAnalysis {
        files: mod_files,
        mod_type,
        warnings,
    })
}

/// Lists every file of `folder`, with warnings for the files that had to be skipped
fn collect_files(folder: &Path) -> Result<(Vec<ArchiveFile>, Vec<String>), InstallationError> {
    let mut files = Vec::new();
    let mut warnings = Vec::new();

    for entry in WalkDir::new(folder) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let absolute = entry.into_path();
        let Some(file_name) = absolute.file_name().and_then(|n| n.to_str()).map(str::to_string) else {
            warnings.push(format!("Skipped '{}': its name isn't valid Unicode", absolute.display()));
            continue;
        };
        if file_name == EXTRACTION_MARKER {
            continue;
        }

        let relative = absolute.strip_prefix(folder).unwrap_or(&absolute).to_path_buf();
        let extension = Path::new(&file_name)
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();

        files.push(ArchiveFile { absolute, relative, file_name, extension });
    }

    Ok((files, warnings))
}

/// Recognizes a game file and computes its destination
fn game_file(file: &ArchiveFile, mod_name: &str) -> Option<ModFile> {
    let mod_type = ModType::of_game_file(&file.extension, &file.file_name)?;
    let destination = mod_type.destination_folder(mod_name, &file.file_name).join(&file.file_name);

    Some(ModFile::new(file, destination, mod_type))
}

/// Recognizes a DLL mod and lists its files
///
/// The folder of the least nested `.dll` is the mod's root: everything inside it is installed
/// in the game root, keeping its relative structure. Game files outside that folder are still
/// installed as usual.
///
/// # Returns
/// * [`Some`] -> The mod contains a `.dll`
/// * [`None`] -> The mod doesn't contain any `.dll`
fn analyse_dll_mod(files: &[ArchiveFile], mod_name: &str) -> Option<Vec<ModFile>> {
    let dll_root = files
        .iter()
        .filter(|f| f.extension == "dll")
        .map(|f| f.relative.parent().unwrap_or(Path::new("")))
        .min_by_key(|folder| folder.components().count())?
        .to_path_buf();

    let mod_files = files
        .iter()
        .filter_map(|f| match f.relative.strip_prefix(&dll_root) {
            Ok(inside_root) if !is_documentation(f) => {
                Some(ModFile::new(f, inside_root.to_path_buf(), ModType::DLL))
            }
            Ok(_) => None,
            Err(_) => game_file(f, mod_name),
        })
        .collect();

    Some(mod_files)
}

/// Whether a file is documentation (readme, license...) rather than part of the mod
fn is_documentation(file: &ArchiveFile) -> bool {
    let name = file.file_name.to_ascii_lowercase();
    DOCUMENTATION_EXTENSIONS.contains(&file.extension.as_str())
        || DOCUMENTATION_PREFIXES.iter().any(|prefix| name.starts_with(prefix))
}
