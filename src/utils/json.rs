//! **json** is a module that contains generic helpers for
//! reading and writing any [`Serialize`]/[`DeserializeOwned`] struct to/from a JSON file
//!
//! This includes:
//! * **loading**: Reading a JSON file into any deserializable type
//! * **saving**: Atomically writing any serializable type out to a JSON file
//!
//! Main functions: [`load_json`], [`save_json`]

use std::fs::{File, create_dir_all, rename};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

/// Reads `path` and deserializes its contents into `T`
///
/// # Arguments
/// * `path` - Path to the JSON file to read
///
/// # Returns
/// * [`Ok`] -> The deserialized `T`
/// * [`Err`] -> [`JsonParsingError::Io`] if the file cannot be opened,
///   or [`JsonParsingError::Json`] if its contents cannot be parsed as valid JSON for `T`
pub fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T, JsonParsingError> {
    let json_file = File::open(path).map_err(|source| JsonParsingError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    serde_json::from_reader(BufReader::new(json_file)).map_err(|source| JsonParsingError::Json {
        path: path.to_path_buf(),
        source,
    })
}

/// Serializes `json_contents` and writes it out to `path`, replacing any existing file
///
/// The contents are first written to a temporary sibling file which then replaces `path`,
/// so a crash (or a full disk) in the middle of a write can never leave a truncated,
/// unreadable data/settings file behind. Missing parent folders are created.
///
/// # Arguments
/// * `path` - Path to write the JSON file to
/// * `json_contents` - The value to serialize
///
/// # Returns
/// * [`Ok`] -> `()` on success
/// * [`Err`] -> [`JsonParsingError::Io`] if the file cannot be created or written,
///   or [`JsonParsingError::Json`] if `json_contents` cannot be serialized
pub fn save_json<T: Serialize>(path: &Path, json_contents: &T) -> Result<(), JsonParsingError> {
    let io_err = |source| JsonParsingError::Io {
        path: path.to_path_buf(),
        source,
    };

    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        create_dir_all(parent).map_err(io_err)?;
    }

    let mut temp_name = path.as_os_str().to_os_string();
    temp_name.push(".tmp");
    let temp_path = PathBuf::from(temp_name);

    let mut writer = BufWriter::new(File::create(&temp_path).map_err(io_err)?);
    serde_json::to_writer_pretty(&mut writer, json_contents).map_err(|source| JsonParsingError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    writer
        .into_inner()
        .map_err(|e| e.into_error())
        .and_then(|file| file.sync_all())
        .map_err(io_err)?;

    rename(&temp_path, path).map_err(io_err)
}

/// Errors that could occur while reading or writing a JSON file
#[derive(Error, Debug)]
pub enum JsonParsingError {
    /// The file could not be opened, created, or written to
    #[error("Couldn't access '{}'. {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    /// The file's contents could not be parsed as, or `T` could not be serialized to, valid JSON
    #[error("'{}' contains invalid JSON. {source}", path.display())]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl JsonParsingError {
    /// Whether the error happened because the file doesn't exist
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::Io { source, .. } if source.kind() == std::io::ErrorKind::NotFound)
    }
}
