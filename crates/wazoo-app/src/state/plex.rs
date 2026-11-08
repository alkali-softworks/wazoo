/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Plex Integration State
 */

use wazoo_core::{PlexServerInfo, PlexSyncProgress, PlexSyncSummary};

#[derive(Debug, Clone, Default)]
pub struct PlexState {
    pub is_testing: bool,
    pub test_result: Option<Result<PlexServerInfo, String>>,
    pub is_syncing: bool,
    pub sync_progress: Option<PlexSyncProgress>,
    pub sync_result: Option<Result<PlexSyncSummary, String>>,
    pub current_sync_id: u64,
}

