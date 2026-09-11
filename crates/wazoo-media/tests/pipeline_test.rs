use std::path::Path;
use wazoo_media::VideoHandle;

#[test]
fn test_video_handle_gstreamer() {
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        let handle = VideoHandle::new(1, sample, "Test Video");
        assert!(handle.is_ok(), "Failed to create VideoHandle: {:?}", handle.err());
        let mut handle = handle.unwrap();

        let dur = handle.duration();
        println!("GStreamer pipeline initialized. Reported duration: {:?}", dur);
        assert_eq!(handle.state.volume, 1.0);

        handle.set_volume(0.5);
        assert_eq!(handle.state.volume, 0.5);

        handle.set_muted(true);
        assert!(handle.state.is_muted);
    }
}

#[test]
fn test_seek_behavior() {
    use std::time::Duration;
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        let mut handle = VideoHandle::new(1, sample, "Test Video").unwrap();
        handle.play();
        std::thread::sleep(Duration::from_millis(500));
        let pos1 = handle.position();
        println!("pos1 (initial ~500ms): {:?}", pos1);
        assert!(pos1 > Duration::ZERO);

        // First seek +5.0s
        handle.seek_relative(5.0);
        let pos_after_seek1 = handle.position();
        println!("pos immediately after +5.0s: {:?}", pos_after_seek1);
        // It must NOT be 0s! It must be near 5.5s
        assert!(pos_after_seek1 >= Duration::from_secs(5), "Expected >= 5s, got {:?}", pos_after_seek1);

        // Immediate second seek +3.0s (accumulates)
        handle.seek_relative(3.0);
        let pos_after_seek2 = handle.position();
        println!("pos immediately after second +3.0s: {:?}", pos_after_seek2);
        assert!(pos_after_seek2 >= Duration::from_secs(8), "Expected >= 8s, got {:?}", pos_after_seek2);

        // Seek backward -5.0s (should be around 3.5s, NOT 0s!)
        handle.seek_relative(-5.0);
        let pos_after_seek3 = handle.position();
        println!("pos immediately after -5.0s: {:?}", pos_after_seek3);
        assert!(pos_after_seek3 >= Duration::from_secs(2), "Expected >= 2s, got {:?}", pos_after_seek3);
        assert!(pos_after_seek3 < Duration::from_secs(6), "Expected < 6s, got {:?}", pos_after_seek3);

        // Seek backward past 0 (should clamp to 0)
        handle.seek_relative(-10.0);
        let pos_clamped = handle.position();
        println!("pos after -10.0s (clamped): {:?}", pos_clamped);
        assert_eq!(pos_clamped, Duration::ZERO);
    }
}

