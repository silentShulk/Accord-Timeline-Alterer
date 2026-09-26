//! **data** contains everything ATA persists or resolves at runtime
//!
//! * [`paths`]: where ATA's files live on the current OS
//! * [`settings`]: user preferences (*settings.json*)
//! * [`mods`]: installed mods (*data.json*)
//! * [`mod_type`]: the kinds of mods ATA understands and where their files go

pub mod mod_type;
pub mod mods;
pub mod paths;
pub mod settings;
