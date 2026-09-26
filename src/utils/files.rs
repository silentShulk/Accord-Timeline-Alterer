//! **files** is a module that contains utility functions for
//! path handling and file system operations shared by every feature
//!
//! This includes:
//! * **extracting**: Pulling filename/parent/stem/extension out of a [`Path`], erroring instead of panicking on missing components or invalid UTF-8
//! * **expanding**: Resolving `~` and shell variables (`$VAR`) inside a path string
//! * **sanitizing**: Turning untrusted relative paths (archive entries, mod names) into safe ones
//! * **moving/comparing**: Cross-device file moves, content comparison, unique names
//! * **cleaning**: Removing the empty folders ATA leaves behind
//!
//! Main functions: [`get_filename_or_err`], [`expand_path`], [`sanitize_relative_path`], [`move_file`], [`prune_empty_dirs`]

use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use shellexpand::full;
use thiserror::Error;

/// Relative folders (from the game root) that must never be removed, even when empty
const PROTECTED_GAME_DIRS: &[&str] = &["data", "data/movie", "wax", "wax/mods"];

/// Name of the folder in which disabled mod files are stored, next to their enabled location
pub const DISABLED_DIR_NAME: &str = ".disabled";

/// Returns the parent directory of `path`, erroring if `path` has none
///
/// # Arguments
/// * `path` - The path to extract the parent from
///
/// # Returns
/// * [`Ok`] -> The parent directory
/// * [`Err`] -> [`FilesInteractionError::ParentlessPath`] if `path` is root or a prefix
pub fn get_parent_or_err(path: &Path) -> Result<&Path, FilesInteractionError> {
    path.parent()
        .ok_or_else(|| FilesInteractionError::ParentlessPath(path.to_path_buf()))
}

/// Returns the filename component of `path` as a UTF-8 string slice
///
/// # Arguments
/// * `path` - The path to extract the filename from
///
/// # Returns
/// * [`Ok`] -> The filename as a `&str`
/// * [`Err`] -> [`FilesInteractionError::NamelessPath`] if `path` has no filename component,
///   or [`FilesInteractionError::InvalidFileName`] if the filename contains invalid Unicode
pub fn get_filename_or_err(path: &Path) -> Result<&str, FilesInteractionError> {
    let name = path
        .file_name()
        .ok_or_else(|| FilesInteractionError::NamelessPath(path.to_path_buf()))?;

    name.to_str()
        .ok_or_else(|| FilesInteractionError::InvalidFileName(name.to_os_string()))
}

/// Returns the filename of `path` with its extension stripped, as a UTF-8 string slice
///
/// # Arguments
/// * `path` - The path to extract the stem from
///
/// # Returns
/// * [`Ok`] -> The file stem as a `&str`
/// * [`Err`] -> [`FilesInteractionError::StemlessPath`] if `path` has no filename component,
///   or [`FilesInteractionError::InvalidStem`] if the stem contains invalid Unicode
pub fn get_filestem_or_err(path: &Path) -> Result<&str, FilesInteractionError> {
    let stem = path
        .file_stem()
        .ok_or_else(|| FilesInteractionError::StemlessPath(path.to_path_buf()))?;

    stem.to_str()
        .ok_or_else(|| FilesInteractionError::InvalidStem(stem.to_os_string()))
}

/// Returns the extension of `path`, lowercased (`"DDS"` and `"dds"` are the same extension)
///
/// # Arguments
/// * `path` - The path to extract the extension from
///
/// # Returns
/// * [`Ok`] -> The lowercase extension
/// * [`Err`] -> [`FilesInteractionError::ExtensionlessPath`] if `path` has no extension,
///   or [`FilesInteractionError::InvalidExtension`] if the extension contains invalid Unicode
pub fn get_extension_or_err(path: &Path) -> Result<String, FilesInteractionError> {
    let extension = path
        .extension()
        .ok_or_else(|| FilesInteractionError::ExtensionlessPath(path.to_path_buf()))?;

    extension
        .to_str()
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| FilesInteractionError::InvalidExtension(extension.to_os_string()))
}

/// Expands `~` and shell variables (e.g. `$HOME`) inside a path string
///
/// # Arguments
/// * `value` - Raw path string, potentially containing `~` or `$VAR` references
///
/// # Returns
/// * [`Ok`] -> Expanded [`PathBuf`]
/// * [`Err`] -> [`FilesInteractionError::EnvExpansion`] if a referenced variable cannot be resolved
pub fn expand_path(value: &str) -> Result<PathBuf, FilesInteractionError> {
    let expanded = full(value)
        .map_err(|e| FilesInteractionError::EnvExpansion(value.to_string(), e.to_string()))?;
    Ok(PathBuf::from(expanded.as_ref()))
}

/// Turns an untrusted relative path (e.g. the name of an archive entry) into a safe relative path
///
/// Both `/` and `\` are treated as separators, `.` components are dropped and
/// anything that could escape the destination folder (`..`, roots, drive letters) is rejected.
///
/// # Returns
/// * [`Some`] -> A relative path made only of normal components
/// * [`None`] -> The path is empty or tries to escape its destination
pub fn sanitize_relative_path(name: &str) -> Option<PathBuf> {
    let mut sanitized = PathBuf::new();

    for part in name.split(['/', '\\']) {
        match part {
            "" | "." => continue,
            ".." => return None,
            // Drive letters/ADS on Windows (`C:`), invalid in a file name anyway
            p if p.contains(':') => return None,
            p => sanitized.push(p),
        }
    }

    // Guard against anything `push` could still interpret as absolute
    let only_normal = sanitized.components().all(|c| matches!(c, Component::Normal(_)));
    (only_normal && !sanitized.as_os_str().is_empty()).then_some(sanitized)
}

/// Checks that `name` can safely be used as a single folder name on every supported OS
///
/// # Returns
/// * `true` if `name` is non-empty and contains no separators, reserved characters
///   or trailing dots/spaces (which Windows silently strips)
pub fn is_valid_folder_name(name: &str) -> bool {
    const RESERVED: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

    !name.trim().is_empty()
        && name != "."
        && name != ".."
        && !name.ends_with(['.', ' '])
        && !name.chars().any(|c| c.is_control() || RESERVED.contains(&c))
}

/// Moves `from` to `to`, falling back to copy + delete when a rename isn't possible
/// (e.g. the two paths are on different drives)
///
/// Missing parent folders of `to` are created.
pub fn move_file(from: &Path, to: &Path) -> io::Result<()> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(rename_err) if from.is_file() => {
            fs::copy(from, to).map_err(|_| rename_err)?;
            fs::remove_file(from)
        }
        Err(rename_err) => Err(rename_err),
    }
}

/// Returns `path` if nothing exists there, otherwise the first free `"<path> (N)"` variant
pub fn unique_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let stem = path.file_stem().unwrap_or_default().to_os_string();
    let extension = path.extension().map(OsStr::to_os_string);

    (1..)
        .map(|n| {
            let mut name = stem.clone();
            name.push(format!(" ({n})"));
            if let Some(ext) = &extension {
                name.push(".");
                name.push(ext);
            }
            path.with_file_name(name)
        })
        .find(|candidate| !candidate.exists())
        .expect("an unbounded range always yields a free name")
}

/// Appends `suffix` to the last component of `path` (`a/b.txt` + `.bak` = `a/b.txt.bak`)
pub fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name: OsString = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

/// Compares the contents of two files
///
/// # Returns
/// * [`Ok`] -> `true` if both files exist and have identical contents
/// * [`Err`] -> One of the files couldn't be read
pub fn files_equal(a: &Path, b: &Path) -> io::Result<bool> {
    if fs::metadata(a)?.len() != fs::metadata(b)?.len() {
        return Ok(false);
    }

    let (mut file_a, mut file_b) = (File::open(a)?, File::open(b)?);
    let (mut buf_a, mut buf_b) = (vec![0u8; 64 * 1024], vec![0u8; 64 * 1024]);

    loop {
        let read = file_a.read(&mut buf_a)?;
        if read == 0 {
            return Ok(true);
        }
        file_b.read_exact(&mut buf_b[..read])?;
        if buf_a[..read] != buf_b[..read] {
            return Ok(false);
        }
    }
}

/// Removes the empty folders left behind after `removed_file` was deleted or moved
///
/// Walks up from the file's folder and deletes every empty folder that is strictly inside
/// `game_root`, stopping at the first non-empty or protected one (e.g. `data/`, `wax/mods/`).
/// For files outside `game_root` only an empty `.disabled/` folder is removed.
/// Failures are ignored: leaving an empty folder behind is never an error.
pub fn prune_empty_dirs(removed_file: &Path, game_root: &Path) {
    let Some(parent) = removed_file.parent() else {
        return;
    };

    if !parent.starts_with(game_root) || game_root.as_os_str().is_empty() {
        if parent.file_name() == Some(OsStr::new(DISABLED_DIR_NAME)) {
            let _ = fs::remove_dir(parent);
        }
        return;
    }

    for dir in parent.ancestors() {
        let Ok(relative) = dir.strip_prefix(game_root) else {
            break;
        };
        let relative = relative.to_string_lossy().replace('\\', "/");
        if relative.is_empty() || PROTECTED_GAME_DIRS.iter().any(|p| p.eq_ignore_ascii_case(&relative)) {
            break;
        }
        // `remove_dir` only succeeds on empty folders
        if fs::remove_dir(dir).is_err() {
            break;
        }
    }
}

/// Errors that could occur while extracting path components or expanding a path string
#[derive(Error, Debug)]
pub enum FilesInteractionError {
    /// The path has no parent (is root or a prefix like `C:\`)
    #[error("{0} does not have a parent (is either root or prefix)")]
    ParentlessPath(PathBuf),

    /// The path has no filename component
    #[error("{0} is a nameless file (path is root or ends in a prefix)")]
    NamelessPath(PathBuf),
    /// The filename component contains invalid Unicode
    #[error("{0:?} contains invalid Unicode characters")]
    InvalidFileName(OsString),

    /// The path has no file stem (no filename, or filename is entirely an extension)
    #[error("{0} is a stemless file (file has no name or has no . at the end)")]
    StemlessPath(PathBuf),
    /// The file stem contains invalid Unicode
    #[error("{0:?} contains invalid Unicode characters")]
    InvalidStem(OsString),

    /// The path has no extension
    #[error("{0} is an extensionless file (file has no name or has no . at the end)")]
    ExtensionlessPath(PathBuf),
    /// The extension contains invalid Unicode
    #[error("{0:?} contains invalid Unicode characters")]
    InvalidExtension(OsString),

    /// A shell variable referenced inside the path could not be resolved
    #[error("Couldn't expand '{0}'. {1}")]
    EnvExpansion(String, String),
}
