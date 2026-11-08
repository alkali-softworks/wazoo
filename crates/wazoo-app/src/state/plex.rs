/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Plex Integration State
 */

use std::collections::HashMap;
use wazoo_core::{
    PlexExplorerItem, PlexExplorerTab, PlexServerInfo, PlexSyncProgress, PlexSyncSummary,
};

#[derive(Debug, Clone, Default)]
pub struct PlexState {
    pub is_testing: bool,
    pub test_result: Option<Result<PlexServerInfo, String>>,
    pub is_syncing: bool,
    pub sync_progress: Option<PlexSyncProgress>,
    pub sync_result: Option<Result<PlexSyncSummary, String>>,
    pub current_sync_id: u64,
    pub live_items: HashMap<PlexExplorerTab, Vec<PlexExplorerItem>>,
    pub is_fetching_live: bool,
}

