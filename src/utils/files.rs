//! **files** is a module that contains utility functions for
//! safely extracting path components and expanding shell paths
//!
//! This includes:
//! * **extracting**: Pulling filename/parent/stem/extension out of a [`Path`], erroring instead of panicking on missing components or invalid UTF-8
//! * **expanding**: Resolving `~` and shell variables (`$VAR`) inside a path string
//!
//! Main functions: [`get_parent_or_err`], [`get_filename_or_err`], [`get_filestem_or_err`], [`get_extension_or_err`], [`expand_path`]

use std::path::{Path, PathBuf};
use std::ffi::OsString;

use thiserror::Error;
use shellexpand::full;



/// Returns the parent directory of `path`, erroring if `path` has none
///
/// # Arguments
/// * `path` - The path to extract the parent from
///
/// # Returns
/// * [`Ok`] -> The parent directory as an owned [`PathBuf`]
/// * [`Err`] -> [`FilesInteractionError::ParentlessPath`] if `path` is root or a prefix
pub fn get_parent_or_err(path: &Path) -> Result<PathBuf, FilesInteractionError> {
    let parent = path.parent().ok_or(FilesInteractionError::ParentlessPath(path.to_path_buf()))?;

    Ok(parent.to_path_buf())
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
    let name = path.file_name().ok_or(FilesInteractionError::NamelessPath(path.to_path_buf()))?;
    let name_str = name.to_str().ok_or(FilesInteractionError::InvalidFileName(name.to_os_string()))?;

    Ok(name_str)
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
    let stem = path.file_stem().ok_or(FilesInteractionError::StemlessPath(path.to_path_buf()))?;
    let stem_str = stem.to_str().ok_or(FilesInteractionError::InvalidStem(stem.to_os_string()))?;

    Ok(stem_str)
}

/// Returns the extension of `path` as a UTF-8 string slice
///
/// # Arguments
/// * `path` - The path to extract the extension from
///
/// # Returns
/// * [`Ok`] -> The extension as a `&str`
/// * [`Err`] -> [`FilesInteractionError::ExtensionlessPath`] if `path` has no extension,
///   or [`FilesInteractionError::InvalidExtension`] if the extension contains invalid Unicode
pub fn get_extension_or_err(path: &Path) -> Result<&str, FilesInteractionError> {
    let extension = path.extension().ok_or(FilesInteractionError::ExtensionlessPath(path.to_path_buf()))?;
    let extension_str = extension.to_str().ok_or(FilesInteractionError::InvalidExtension(extension.to_os_string()))?;

    Ok(extension_str)
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