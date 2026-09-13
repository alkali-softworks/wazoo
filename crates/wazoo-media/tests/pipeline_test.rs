/*!
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
fn test_video_dimensions_and_aspect_ratio() {
    let sample = "/home/klo/Downloads/Cat_s_Evil_Supervillain_Laugh.mp4";
    if Path::new(sample).exists() {
        if let Ok(mut handle) = VideoHandle::new(1, sample, "Test Video") {
            // Pump frames to ensure metadata is loaded
            for _ in 0..10 {
                handle.update_frame();
                std::thread::sleep(Duration::from_millis(20));
            }
            let dims = handle.video_dimensions();
            assert_eq!(dims, Some((720, 1280)));
            let ar = handle.aspect_ratio();
            assert!(ar.is_some());
            let ratio = ar.unwrap();
            assert!((ratio - 720.0 / 1280.0).abs() < 0.01);
        }
    }

    let sample_16_9 = "/home/klo/Downloads/amilia.mp4";
    if Path::new(sample_16_9).exists() {
        if let Ok(mut handle) = VideoHandle::new(2, sample_16_9, "16:9 Video") {
            for _ in 0..10 {
                handle.update_frame();
                std::thread::sleep(Duration::from_millis(20));
            }
            let dims = handle.video_dimensions();
            assert_eq!(dims, Some((640, 360)));
            let ar = handle.aspect_ratio();
            assert!(ar.is_some());
            let ratio = ar.unwrap();
            assert!((ratio - 16.0 / 9.0).abs() < 0.01);
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
