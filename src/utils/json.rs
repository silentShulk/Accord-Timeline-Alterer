//! **json** is a module that contains generic helpers for
//! reading and writing any [`Serialize`]/[`DeserializeOwned`] struct to/from a JSON file
//!
//! This includes:
//! * **loading**: Reading a JSON file into any deserializable type
//! * **saving**: Writing any serializable type out to a JSON file
//!
//! Main functions: [`load_json`], [`save_json`]

use std::fs::File;
use std::path::Path;
use std::io::BufReader;

use thiserror::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;



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
    let json_file = File::open(path)?;
    let reader = BufReader::new(json_file);

    Ok(serde_json::from_reader(reader)?)
}

/// Serializes `json_contents` and writes it out to `path`, overwriting any existing file
///
/// # Arguments
/// * `path` - Path to write the JSON file to
/// * `json_contents` - The value to serialize
///
/// # Returns
/// * [`Ok`] -> `()` on success
/// * [`Err`] -> [`JsonParsingError::Io`] if the file cannot be created or written,
///   or [`JsonParsingError::Json`] if `json_contents` cannot be serialized
pub fn save_json<T: Serialize>(path: &Path, json_contents: &T) -> Result<(), JsonParsingError>{
    let json_file = File::create(path)?;
    serde_json::to_writer_pretty(json_file, json_contents)?;

    Ok(())
}

/// Errors that could occur while reading or writing a JSON file
#[derive(Error, Debug)]
pub enum JsonParsingError {
    /// The file could not be opened, created, or written to
    #[error("IO error. {0}")]
    Io(#[from] std::io::Error),

    /// The file's contents could not be parsed as, or `T` could not be serialized to, valid JSON
    #[error("Json parsing error. {0}")]
    Json(#[from] serde_json::Error),
}