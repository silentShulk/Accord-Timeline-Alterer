use crate::settings::Settings;
use std::path::PathBuf;

use crate::settings::{Style, Palette, SortingOrder, ConflictResolution};



fn set_style(settings: &mut Settings, style: Style) -> Result<Settings, crate::settings::SettingsInteractionError> {
    settings.style = style;
    settings.update_settings_file()?;

    Ok(settings.clone())
}
fn set_palette(settings: &mut Settings, palette: Palette) -> Result<Settings, crate::settings::SettingsInteractionError> {
    settings.palette = palette;
    settings.update_settings_file()?;

    Ok(settings.clone())
}
fn set_sorting_order(settings: &mut Settings, sorting_order: SortingOrder) -> Result<Settings, crate::settings::SettingsInteractionError> {
    settings.sorting_order = sorting_order;
    settings.update_settings_file()?;

    Ok(settings.clone())
}
fn set_files_conflict_resolution(settings: &mut Settings, conflict_resolution: ConflictResolution) -> Result<Settings, crate::settings::SettingsInteractionError> {
    settings.files_conflict_resolution = conflict_resolution;
    settings.update_settings_file()?;

    Ok(settings.clone())
}
fn set_keep_extracted_folders(settings: &mut Settings, keep_extracted_folders: bool) -> Result<Settings, crate::settings::SettingsInteractionError> {
    settings.keep_extracted_folders = keep_extracted_folders;
    settings.update_settings_file()?;

    Ok(settings.clone())
}
fn set_extracted_folders_location(settings: &mut Settings, extracted_folders_location: PathBuf) -> Result<Settings, crate::settings::SettingsInteractionError> {
    settings.extracted_folders_location = extracted_folders_location;
    settings.update_settings_file()?;

    Ok(settings.clone())
}
fn set_game_path(settings: &mut Settings, game_path: PathBuf) -> Result<Settings, crate::settings::SettingsInteractionError> {
    settings.game_path = game_path;
    settings.update_settings_file()?;

    Ok(settings.clone())
}
fn set_discord_rich_presence(settings: &mut Settings, discord_rich_presence: String) -> Result<Settings, crate::settings::SettingsInteractionError> {
    settings.discord_rich_presence = discord_rich_presence;
    settings.update_settings_file()?;

    Ok(settings.clone())
}