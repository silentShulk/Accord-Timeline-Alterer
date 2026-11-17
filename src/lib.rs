//! **ata** is the core library behind ATA (Accord's Timeline Alterer),
//! the cross-platform NieR:Automata mod manager.
//!
//! This includes:
//! * **data**: persisted state ([`data::mods`], [`data::settings`], [`data::paths`])
//! * **features**: the operations on that state ([`features::installation`],  [`features::uninstallation`], [`features::mod_managing`], [`features::misc`])
//! * **utils**: small shared helpers for files and JSON
//!
//! The library never prints to stdout/stderr: everything is reported through
//! return values, so callers decide how to show it.

pub mod data;
pub mod features;
pub mod utils;
