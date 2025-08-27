pub static RECENTLY_USED: &str = "Recently Used";
pub static COLLECTION: &str = "Collection";
pub static SEARCH: &str = "Search";

// UI Icon constants - better to use constants than hardcoded characters
pub const CANCELLATION: char = '🗙';
pub const COG_WHEEL: char = '⚙';
pub const HAMBURGER: char = '☰';
pub const MAGNIFIER: char = '🔍';
pub const NAME_BADGE: char = '📛';
pub const LOWER_UPPER_CASE: char = '🗛';
pub const PUSH_PIN: char = '📌';
pub const SUBSET: char = '⊂';

pub fn recently_used_id() -> egui::Id {
    egui::Id::new(RECENTLY_USED)
}

pub fn collection_id() -> egui::Id {
    egui::Id::new(COLLECTION)
}

pub fn search_id() -> egui::Id {
    egui::Id::new(SEARCH)
}
