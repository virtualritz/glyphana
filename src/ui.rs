pub static RECENTLY_USED: &str = "Recently Used";
pub static COLLECTION: &str = "Collection";
pub static SEARCH: &str = "Search";

pub fn recently_used_id() -> egui::Id {
    egui::Id::new(RECENTLY_USED)
}

pub fn collection_id() -> egui::Id {
    egui::Id::new(COLLECTION)
}

pub fn search_id() -> egui::Id {
    egui::Id::new(SEARCH)
}
