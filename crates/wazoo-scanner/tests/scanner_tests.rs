use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use wazoo_scanner::{ScanStage, Scanner, clean_video_name, is_video_file, probe_video_metadata};

#[test]
fn test_clean_video_name() {
    assert_eq!(
        clean_video_name("[1080p] My_Great.Movie-Part1"),
        "My Great Movie-Part1"
    );
    assert_eq!(
        clean_video_name("[Group] Nature_Documentary_[HEVC]"),
        "Nature Documentary"
    );
}

#[test]
fn test_is_video_file() {
    assert!(is_video_file("test.mp4"));
    assert!(is_video_file("test.mkv"));
    assert!(is_video_file("test.MKV"));
    assert!(!is_video_file("test.txt"));
    assert!(!is_video_file("test.png"));
}

#[tokio::test]
async fn test_scanner_no_ffprobe() {
    let tmp = std::env::temp_dir().join(format!(
        "wazoo_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&tmp);
    let test_file = tmp.join("Test.Video.2026.mkv");
    let _ = std::fs::write(&test_file, b"dummy video content");
    let db_file = tmp.join("test.db");

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let scanner = Scanner::default();
    let folders = vec![tmp.to_string_lossy().to_string()];
    let res = scanner.scan_and_index(&folders, db_file, Some(tx)).await;
    assert_eq!(res.unwrap(), 1);

    let mut progress_count = 0;
    while let Ok(progress) = rx.try_recv() {
        progress_count += 1;
        assert!(progress.total <= 1);
    }
    assert!(progress_count >= 1);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn test_scanner_listing_percentage() {
    let tmp = std::env::temp_dir().join(format!(
        "wazoo_test_pct_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let dir_a = tmp.join("Anime").join("ShowA");
    let dir_b = tmp.join("Anime").join("ShowB");
    let _ = std::fs::create_dir_all(&dir_a);
    let _ = std::fs::create_dir_all(&dir_b);
    let _ = std::fs::write(dir_a.join("Ep1.mkv"), b"video1");
    let _ = std::fs::write(dir_a.join("Ep2.mp4"), b"video2");
    let _ = std::fs::write(dir_b.join("Ep1.mkv"), b"video3");
    let db_file = tmp.join("test.db");

    let (tx, mut rx) = tokio::sync::mpsc::channel(20);
    let scanner = Scanner::default();
    let folders = vec![tmp.join("Anime").to_string_lossy().to_string()];
    let res = scanner.scan_and_index(&folders, db_file, Some(tx)).await;
    assert_eq!(res.unwrap(), 3);

    let mut listing_stages = Vec::new();
    while let Ok(progress) = rx.try_recv() {
        if matches!(progress.stage, ScanStage::Listing) {
            listing_stages.push(progress);
        }
    }

    assert!(
        !listing_stages.is_empty(),
        "Should have received Listing progress events"
    );
    // Verify we get percentage and file count
    let last_listing = listing_stages.last().unwrap();
    assert_eq!(last_listing.percent, 100);
    assert_eq!(last_listing.files_found, 3);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn test_scanner_cancellation() {
    let tmp = std::env::temp_dir().join(format!(
        "wazoo_test_cancel_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let dir = tmp.join("Videos");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("video.mp4"), b"dummy content");
    let db_file = tmp.join("test.db");

    let cancel = Arc::new(AtomicBool::new(true));
    let scanner = Scanner::default();
    let folders = vec![dir.to_string_lossy().to_string()];

    let res = scanner
        .scan_and_index_with_cancel(&folders, db_file, None, Some(cancel))
        .await;
    assert!(res.is_err());
    assert_eq!(res.err().as_deref(), Some("Scan cancelled"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_probe_dash_filename() {
    // Filenames starting with '-' must not cause ffprobe option parsing errors
    let meta = probe_video_metadata("-option_like_name.mp4", None);
    assert_eq!(meta.codec, "unknown");
    assert_eq!(meta.width, 0);
}
