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
