//! **listing** is a module that sorts the installed mods for display
//!
//! Main function: [`list_mods`]

use crate::data::mods::Mod;
use crate::data::settings::SortingOrder;

use std::cmp::Reverse;

/// Returns the mods sorted according to the requested [`SortingOrder`]
///
/// Sorting is stable: mods that compare equal keep their installation order.
///
/// # Arguments
/// * `sorting_order` - Criterion used to sort the mod list
/// * `mods` - The mods to sort
///
/// # Returns
/// * References to `mods`, sorted based on `sorting_order`
pub fn list_mods(sorting_order: SortingOrder, mods: &[Mod]) -> Vec<&Mod> {
    let mut sorted: Vec<&Mod> = mods.iter().collect();

    match sorting_order {
        SortingOrder::ModType => sorted.sort_by_key(|m| m.mod_type),
        SortingOrder::InstallDate => sorted.sort_by_key(|m| Reverse(m.install_date)),
        SortingOrder::EnableStatus => sorted.sort_by_key(|m| Reverse(m.enabled)),
        SortingOrder::Alphabetical => sorted.sort_by_cached_key(|m| m.name.to_lowercase()),
        SortingOrder::Size => sorted.sort_by_cached_key(|m| Reverse(m.total_size())),
    }

    sorted
}
