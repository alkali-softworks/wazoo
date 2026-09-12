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

#[test]
fn test_video_handle_auto_finish() {
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        if let Ok(mut handle) = VideoHandle::new(1, sample, "Test Video") {
            let dur = handle.duration();
            if dur > Duration::from_millis(600) {
                handle.seek(dur - Duration::from_millis(300));
            }
            std::thread::sleep(Duration::from_millis(1100));
            let start = std::time::Instant::now();
            let mut finished = false;
            while start.elapsed() < Duration::from_secs(4) {
                std::thread::sleep(Duration::from_millis(30));
                handle.update_frame();
                if handle.is_finished() {
                    finished = true;
                    break;
                }
            }
            assert!(finished, "Video handle must detect completion at end of file");
        }
    }
}
