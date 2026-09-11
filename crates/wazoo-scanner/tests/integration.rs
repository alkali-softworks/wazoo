use std::path::Path;
use wazoo_scanner::probe_video_metadata;

#[test]
fn test_probe_real_video() {
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        let meta = probe_video_metadata(sample, None);
        println!("Probed metadata: {:?}", meta);
        assert!(!meta.codec.is_empty());
        assert!(meta.duration > 0.0);
    }
}
