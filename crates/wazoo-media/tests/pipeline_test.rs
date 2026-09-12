/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Media Pipeline Integration Tests
 */

use std::path::Path;
use std::time::Duration;
use wazoo_media::VideoHandle;

#[test]
fn test_watchdog_threshold_constant() {
    assert_eq!(VideoHandle::STUCK_THRESHOLD_SECONDS, 30);
    assert_eq!(VideoHandle::SEEK_GRACE_PERIOD, Duration::from_secs(10));
}

#[test]
fn test_video_handle_lifecycle() {
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        if let Ok(mut handle) = VideoHandle::new(1, sample, "Test Video") {
            handle.set_muted(true);
            assert!(handle.state.is_muted);
            assert!(handle.duration() > Duration::ZERO);
        }
    }
}
