//! **ATA** (Accord's Timeline Alterer) is a mod manager for NieR:Automata
//!
//! This crate is the whole backend of ATA. It is used in two ways:
//! * as a **library**, by Rust GUIs (ShellUI) and by the Launcher
//! * as a **CLI** (the `ATA` executable, see `main.rs`), by GUIs written with any other technology
//!
//! Layout:
//! * [`data`]: persisted state (installed mods, settings, application paths)
//! * [`features`]: what ATA can do (install, uninstall, enable/disable, list, game helpers)
//! * [`utils`]: generic helpers (path handling, JSON files, file moving)
//!
//! Cargo features:
//! * `archives` (default): mod installation from `.zip`/`.7z`/`.rar` archives
//! * `cli` (default): the `ATA` executable
//! * `specta`: derives `specta::Type` on public data types, for TypeScript bindings generation

pub mod data;
pub mod features;
pub mod utils;

pub use data::mod_type::ModType;
pub use data::mods::{Backup, Mod, Mods};
pub use data::paths::{PATHS, Paths};
pub use data::settings::{ConflictResolution, Palette, Settings, SortingOrder};
