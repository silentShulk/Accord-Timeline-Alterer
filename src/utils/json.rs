//! **json** is a module that contains generic helpers for
//! reading and writing any [`Serialize`]/[`DeserializeOwned`] struct to/from a JSON file
//!
//! This includes:
//! * **loading**: Reading a JSON file into any deserializable type
//! * **saving**: Writing any serializable type out to a JSON file
//!
//! Main functions: [`load_json`], [`save_json`]

use crate::utils::files::expand_path;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::BufReader;

use thiserror::Error;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};



/// Serializes `value` to a compact JSON string and returns it
///
/// Panics if `value` cannot be serialized — this should never happen for
/// the types used in this codebase.
///
/// # Arguments
/// * `value` - Any value that implements [`serde::Serialize`]
///
/// # Returns
/// * A [`String`] containing the compact JSON representation of `value`
pub fn json<T>(value: &T) -> String
where
    T: serde::Serialize,
{
    serde_json::to_string(value).unwrap()
}

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

/// Deserializes a string path and expands it (e.g., resolving `~` or environment variables) into a [`PathBuf`][cite: 1]
///
/// Intended for use with `#[serde(deserialize_with = "deserialize_expanded_path")]` on struct fields.
///
/// # Arguments
/// * `deserializer` - The Serde [`Deserializer`] providing the raw path string[cite: 1]
///
/// # Returns
/// * [`Ok`] -> The expanded [`PathBuf`][cite: 1]
/// * [`Err`] -> `D::Error` if deserializing the string fails or [`expand_path`] returns an error[cite: 1]
pub fn deserialize_expanded_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    expand_path(&raw).map_err(serde::de::Error::custom)
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