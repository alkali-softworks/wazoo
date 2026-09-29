/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Media Scanner State
 */

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use wazoo_scanner::ScanProgress;

#[derive(Debug, Clone, Default)]
pub struct ScannerState {
    pub is_scanning: bool,
    pub current_scan_id: u64,
    pub scan_cancel: Option<Arc<AtomicBool>>,
    pub scan_progress: Option<ScanProgress>,
}
