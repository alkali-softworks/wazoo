/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Search Queries & Folder Filters State
 */

/// Manages active search queries, tokenized tags, and folder filter selections.
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub active_query: String,
    pub input: String,
    pub tags: Vec<String>,
    pub folder_input: String,
    pub active_folders: Vec<String>,
    pub selected_folders: Vec<String>,
    pub active_folder: String,
    pub selected_folder: String,
}

impl SearchState {
    /// Returns true if an active non-empty search query filter is currently applied.
    #[inline]
    pub fn is_query_active(&self) -> bool {
        !self.active_query.trim().is_empty()
    }
}
