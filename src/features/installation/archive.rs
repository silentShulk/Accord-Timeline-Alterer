//! **archive** turns what the user picked (an archive, or an already extracted folder)
//! into a folder ATA can analyse
//!
//! Archives are extracted inside [`crate::Settings::extraction_dir`]. Entries trying to
//! escape the extraction folder (`../`, absolute paths) are skipped. A folder previously
//! extracted by ATA is reused, but a folder ATA didn't create is never deleted: a new
//! name is picked instead.
//!
//! Main function: [`prepare_source`]

use super::InstallationError;
use crate::utils::files::{get_extension_or_err, get_filestem_or_err, sanitize_relative_path, unique_path};

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

/// Marker file identifying a folder created by ATA's extraction (safe to delete/reuse)
pub(super) const EXTRACTION_MARKER: &str = ".ata-extracted";

/// Archive formats ATA can extract
pub const SUPPORTED_ARCHIVES: &[&str] = &["zip", "7z", "rar"];

/// A folder containing the files of the mod to install
///
/// If it was extracted by ATA and must not be kept, it is deleted when dropped,
/// whether the installation succeeded or not.
pub(super) struct ModSource {
    path: PathBuf,
    /// Whether the folder was created by ATA (its files can be moved instead of copied)
    extracted: bool,
    keep: bool,
}

impl ModSource {
    /// Folder containing the mod files
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether files can be moved out of the folder instead of copied (faster, same result)
    pub fn is_disposable(&self) -> bool {
        self.extracted && !self.keep
    }
}

impl Drop for ModSource {
    fn drop(&mut self) {
        if self.extracted && !self.keep {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

/// Makes the mod at `source` available as a folder
///
/// # Arguments
/// * `source` - An archive (`.zip`, `.7z`, `.rar`) or a folder
/// * `extraction_dir` - Where archives get extracted
/// * `keep_extracted` - Whether the extracted folder must survive the installation
///
/// # Errors
/// * [`InstallationError::FilesInteraction`] if the archive path has no extension or filestem
/// * [`InstallationError::UnsupportedCompression`] if the archive type is unsupported
/// * [`InstallationError::FileManaging`] if the extraction folder can't be created
/// * [`InstallationError::ZipExtraction`], [`InstallationError::SevenZipExtraction`],
///   [`InstallationError::RarExtraction`] if the archive can't be extracted
pub(super) fn prepare_source(source: &Path, extraction_dir: &Path, keep_extracted: bool) -> Result<ModSource, InstallationError> {
    if source.is_dir() {
        return Ok(ModSource {
            path: source.to_path_buf(),
            extracted: false,
            keep: true,
        });
    }

    let extension = get_extension_or_err(source)?;
    if !SUPPORTED_ARCHIVES.contains(&extension.as_str()) {
        return Err(InstallationError::UnsupportedCompression(source.to_path_buf()));
    }

    let target = extraction_target(&extraction_dir.join(get_filestem_or_err(source)?))?;
    fs::create_dir_all(&target)?;
    File::create(target.join(EXTRACTION_MARKER))?;

    // Until the extraction succeeds, the folder is always cleaned up on failure
    let mut extracted = ModSource {
        path: target,
        extracted: true,
        keep: false,
    };

    match extension.as_str() {
        "zip" => extract_zip(source, extracted.path())?,
        "7z" => extract_7z(source, extracted.path())?,
        _ => extract_rar(source, extracted.path())?,
    }

    extracted.keep = keep_extracted;
    Ok(extracted)
}

/// Picks the folder an archive gets extracted into
///
/// A leftover extraction of ATA is deleted and reused; any other existing folder
/// belongs to the user and is left untouched (a free `"<name> (N)"` is used instead).
fn extraction_target(wanted: &Path) -> io::Result<PathBuf> {
    if !wanted.exists() {
        Ok(wanted.to_path_buf())
    } else if wanted.join(EXTRACTION_MARKER).is_file() {
        fs::remove_dir_all(wanted)?;
        Ok(wanted.to_path_buf())
    } else {
        Ok(unique_path(wanted))
    }
}

/// Extracts a ZIP archive (the zip crate already rejects entries escaping `destination`)
fn extract_zip(archive: &Path, destination: &Path) -> Result<(), InstallationError> {
    let mut zip_archive = zip::ZipArchive::new(File::open(archive)?)?;
    zip_archive.extract(destination)?;
    Ok(())
}

/// Extracts a 7z archive, skipping entries whose path would escape `destination`
fn extract_7z(archive: &Path, destination: &Path) -> Result<(), InstallationError> {
    sevenz_rust::decompress_file_with_extract_fn(archive, destination, |entry, reader, _unsafe_dest| {
        match sanitize_relative_path(entry.name()) {
            Some(relative) => sevenz_rust::default_entry_extract_fn(entry, reader, &destination.join(relative)),
            None => {
                // Entries of solid archives share a stream: skipped data must still be consumed
                io::copy(reader, &mut io::sink()).map_err(sevenz_rust::Error::io)?;
                Ok(true)
            }
        }
    })?;
    Ok(())
}

/// Extracts a RAR archive entry by entry, skipping entries whose path would escape `destination`
fn extract_rar(archive: &Path, destination: &Path) -> Result<(), InstallationError> {
    let mut rar_archive = unrar::Archive::new(archive).open_for_processing()?;

    while let Some(header) = rar_archive.read_header()? {
        let entry = header.entry();
        let relative = entry
            .is_file()
            .then(|| sanitize_relative_path(&entry.filename.to_string_lossy()))
            .flatten();

        rar_archive = match relative {
            Some(relative) => {
                let file = destination.join(relative);
                if let Some(parent) = file.parent() {
                    fs::create_dir_all(parent)?;
                }
                // `extract_to` writes the entry to exactly this file path
                header.extract_to(&file)?
            }
            None => header.skip()?,
        };
    }

    Ok(())
}
