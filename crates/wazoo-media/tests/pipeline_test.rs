/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Media Pipeline Integration Tests
 * 
 * Validates playback initialization, buffering cache options, random seeking behavior,
 * and codec decoding capabilities.
 */

use std::path::Path;
use std::time::Duration;
use wazoo_media::{BufferConfig, VideoHandle};

#[test]
fn test_video_handle_mpv() {
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        let handle = VideoHandle::new(1, sample, "Test Video");
        assert!(handle.is_ok(), "Failed to create VideoHandle: {:?}", handle.err());
        let mut handle = handle.unwrap();

        // Ensure muted immediately so tests remain completely silent
        handle.set_muted(true);
        assert!(handle.state.is_muted);

        let dur = handle.duration();
        println!("libmpv initialized. Reported duration: {:?}", dur);
        assert!(dur > Duration::ZERO);

        handle.set_volume(0.5);
        assert_eq!(handle.state.volume, 0.5);

        handle.set_muted(false);
        assert!(!handle.state.is_muted);
        handle.set_muted(true);

        handle.set_subtitles_visible(true);
        handle.set_subtitles_visible(false);

        // Start playback and verify update_frame() produces a frame
        handle.play();
        std::thread::sleep(Duration::from_millis(200));
        let frame = handle.update_frame();
        println!("update_frame returned new frame: {frame}");
        let (w, h) = handle.dimensions();
        println!("video dimensions: {w}x{h}");
        assert_eq!(w, 1280);
        assert_eq!(h, 720);
    }
}

#[test]
fn test_mpv_buffering_and_cache() {
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        let handle = VideoHandle::with_buffering(
            2,
            sample,
            "Buffer Test Video",
            BufferConfig {
                duration_secs: 10,
                size_mb: 64,
                read_chunk_kb: 512,
                preferred_audio_language: None,
            },
        );
        assert!(handle.is_ok(), "Failed to create buffered VideoHandle: {:?}", handle.err());
        let mut handle = handle.unwrap();
        handle.set_muted(true);

        let dur = handle.duration();
        assert!(dur > Duration::ZERO);

        handle.play();
        std::thread::sleep(Duration::from_millis(150));
        let _ = handle.update_frame();
        let pos = handle.position();
        println!("Buffered handle position: {:?}", pos);
    }
}

#[test]
fn test_seek_behavior() {
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        let mut handle = VideoHandle::new(1, sample, "Test Video").unwrap();
        handle.set_muted(true);
        handle.play();
        std::thread::sleep(Duration::from_millis(300));
        let pos1 = handle.position();
        println!("pos1 (initial ~300ms): {:?}", pos1);
        assert!(pos1 > Duration::ZERO);

        // First seek +5.0s
        handle.seek_relative(5.0);
        let pos_after_seek1 = handle.position();
        println!("pos immediately after +5.0s: {:?}", pos_after_seek1);
        // It must NOT be 0s! It must be near 5s+
        assert!(pos_after_seek1 >= Duration::from_secs(4), "Expected >= 4s, got {:?}", pos_after_seek1);

        // Immediate second seek +3.0s (accumulates)
        handle.seek_relative(3.0);
        let pos_after_seek2 = handle.position();
        println!("pos immediately after second +3.0s: {:?}", pos_after_seek2);
        assert!(pos_after_seek2 >= Duration::from_secs(7), "Expected >= 7s, got {:?}", pos_after_seek2);

        // Seek backward -5.0s
        handle.seek_relative(-5.0);
        let pos_after_seek3 = handle.position();
        println!("pos immediately after -5.0s: {:?}", pos_after_seek3);
        assert!(pos_after_seek3 >= Duration::from_secs(1), "Expected >= 1s, got {:?}", pos_after_seek3);
        assert!(pos_after_seek3 < Duration::from_secs(6), "Expected < 6s, got {:?}", pos_after_seek3);

        // Seek backward past 0 (should clamp to 0)
        handle.seek_relative(-10.0);
        let pos_clamped = handle.position();
        println!("pos after -10.0s (clamped): {:?}", pos_clamped);
        assert_eq!(pos_clamped, Duration::ZERO);

        // Verify check_stuck() is false during seek grace period
        assert!(!handle.check_stuck(), "check_stuck should not trigger during seek grace period");

        // Verify check_stuck() is false when paused
        handle.pause();
        assert!(!handle.check_stuck(), "check_stuck should never trigger when paused");
        handle.play();

        // Forward seek way past duration should NOT immediately trigger is_finished()
        let dur = handle.duration();
        handle.seek_relative(100.0);
        let pos_near_end = handle.position();
        println!("pos after seeking way past end: {:?} (duration: {:?})", pos_near_end, dur);
        assert!(pos_near_end < dur, "Position should be clamped before duration, got {:?}", pos_near_end);
        assert!(!handle.is_finished(), "is_finished() must NOT trigger immediately after a seek!");
        assert!(!handle.check_stuck(), "check_stuck() must NOT trigger immediately after a seek!");
    }
}

#[test]
fn test_watchdog_threshold_constant() {
    assert_eq!(VideoHandle::STUCK_THRESHOLD_SECONDS, 30);
    assert_eq!(VideoHandle::SEEK_GRACE_PERIOD, Duration::from_secs(10));
}

#[test]
fn test_av1_ninkoro_playback() {
    let sample = "/mnt/bob/anime/NinKoro/NinKoro - 01.mkv";
    if Path::new(sample).exists() {
        println!("Testing 10-bit AV1 anime playback via libmpv on {}", sample);
        let handle = VideoHandle::new(3, sample, "NinKoro 01");
        assert!(handle.is_ok(), "Failed to open NinKoro with libmpv: {:?}", handle.err());
        let mut handle = handle.unwrap();

        // Always mute during test
        handle.set_muted(true);
        handle.play();

        // Let it render frames of 10-bit AV1 with ASS subtitles (give network mount enough time to buffer)
        let mut rendered_frames = 0;
        for _ in 0..50 {
            std::thread::sleep(Duration::from_millis(100));
            if handle.update_frame() {
                rendered_frames += 1;
                if rendered_frames >= 3 {
                    break;
                }
            }
        }
        println!("NinKoro rendered {} frames smoothly with libmpv", rendered_frames);
        assert!(rendered_frames > 0, "Expected at least 1 rendered frame");
        assert!(handle.duration() > Duration::from_secs(60), "Duration should be anime length (> 60s)");

        // Verify that persistent shader view element can be created
        let _element = handle.view::<()>(1.0);
    }
}

#[test]
fn test_seek_immediately_after_init() {
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        let mut handle = VideoHandle::new(2, sample, "Test Video 2").unwrap();
        handle.set_muted(true);
        handle.seek(Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(600));
        let _ = handle.update_frame();
        let pos = handle.position();
        println!("Position after immediate seek and 600ms sleep: {:?}", pos);
        assert!(pos >= Duration::from_secs(9), "Expected pos >= 9s, but got {:?}", pos);
    }
}

#[test]
fn test_loadfile_start_option() {
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        let buffer_config = wazoo_media::BufferConfig {
            duration_secs: 10,
            size_mb: 64,
            read_chunk_kb: 512,
            preferred_audio_language: None,
        };
        let mut handle = VideoHandle::with_buffering_and_start(
            3,
            sample,
            "Test Video 3",
            buffer_config,
            Some(12.5),
        )
        .unwrap();
        assert_eq!(handle.position(), Duration::from_secs_f64(12.5));

        // Wait a bit, tick frame, and verify position stays >= 12.0s
        std::thread::sleep(Duration::from_millis(600));
        let _ = handle.update_frame();
        let pos = handle.position();
        println!("Position after with_buffering_and_start(12.5): {:?}", pos);
        assert!(pos >= Duration::from_secs(11), "Expected pos >= 11s, got {:?}", pos);
    }
}

#[test]
fn test_audio_stream_switching() {
    let sample = "/mnt/bob/anime/Mushoku Tensei/Season 2/S02E14-Wedding Reception.mkv";
    if Path::new(sample).exists() {
        let mut handle = VideoHandle::new(10, sample, "Mushoku Tensei").expect("create VideoHandle");
        handle.set_muted(true);

        // Sleep briefly and update frame to process libmpv events
        std::thread::sleep(Duration::from_millis(300));
        let _ = handle.update_frame();

        let tracks = handle.audio_tracks();
        println!("Found {} audio tracks", tracks.len());
        for (i, t) in tracks.iter().enumerate() {
            println!("  [{}] id={} selected={} lang={:?} title={:?} label='{}'",
                i, t.id, t.is_selected, t.lang, t.title, wazoo_media::format_audio_track_label(t, i)
            );
        }

        assert_eq!(tracks.len(), 2, "Expected 2 audio tracks for Wedding Reception");
        assert_eq!(wazoo_media::format_audio_track_label(&tracks[0], 0), "English Dub");
        assert_eq!(wazoo_media::format_audio_track_label(&tracks[1], 1), "Japanese");

        // Initial aid should be 1
        assert_eq!(handle.current_audio_track_id(), Some(1));

        // Switch to Japanese (id 2)
        handle.set_audio_track(2);
        assert_eq!(handle.current_audio_track_id(), Some(2));
        assert!(handle.audio_tracks().iter().find(|t| t.id == 2).unwrap().is_selected);
        assert!(!handle.audio_tracks().iter().find(|t| t.id == 1).unwrap().is_selected);

        // Switch back to English Dub (id 1)
        handle.set_audio_track(1);
        assert_eq!(handle.current_audio_track_id(), Some(1));
        assert!(handle.audio_tracks().iter().find(|t| t.id == 1).unwrap().is_selected);
    }
}

#[test]
fn test_audio_stream_auto_selection() {
    let sample = "/mnt/bob/anime/Mushoku Tensei/Season 2/S02E14-Wedding Reception.mkv";
    if Path::new(sample).exists() {
        let config = BufferConfig {
            duration_secs: 10,
            size_mb: 32,
            read_chunk_kb: 512,
            preferred_audio_language: Some("Japanese".to_string()),
        };
        let mut handle = VideoHandle::with_buffering(11, sample, "Mushoku Tensei", config)
            .expect("create VideoHandle with preferred Japanese audio");
        handle.set_muted(true);

        std::thread::sleep(Duration::from_millis(300));
        let _ = handle.update_frame();

        // Track 2 (Japanese) should be automatically selected upon loading!
        assert_eq!(handle.current_audio_track_id(), Some(2));
        assert!(handle.audio_tracks().iter().find(|t| t.id == 2).unwrap().is_selected);
        assert!(!handle.audio_tracks().iter().find(|t| t.id == 1).unwrap().is_selected);
    }
}
#[test]
fn test_subtitle_tracks_and_extraction() {
    let sample = "/mnt/bob/anime/Mushoku Tensei/Season 2/S02E14-Wedding Reception.mkv";
    if Path::new(sample).exists() {
        let mut handle = VideoHandle::new(12, sample, "Mushoku Tensei").expect("create VideoHandle");
        handle.set_muted(true);

        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(100));
            let _ = handle.update_frame();
            if handle.subtitle_tracks().len() >= 3 {
                break;
            }
        }

        let sub_tracks = handle.subtitle_tracks().to_vec();
        println!("Found {} subtitle tracks", sub_tracks.len());
        for (i, t) in sub_tracks.iter().enumerate() {
            println!("  [{}] id={} selected={} lang={:?} title={:?} label='{}'",
                i, t.id, t.is_selected, t.lang, t.title, wazoo_media::format_subtitle_track_label(t, i)
            );
        }

        assert!(sub_tracks.len() >= 3, "Expected at least 3 subtitle tracks for Wedding Reception");
        assert_eq!(wazoo_media::format_subtitle_track_label(&sub_tracks[0], 0), "English (Full Subtitles)");
        assert_eq!(wazoo_media::format_subtitle_track_label(&sub_tracks[1], 1), "English (Full Subtitles - Honorifics)");
        assert_eq!(wazoo_media::format_subtitle_track_label(&sub_tracks[2], 2), "English (Signs/Songs)");

        // Subtitle switching
        handle.set_subtitle_track(2);
        assert_eq!(handle.current_subtitle_track_id(), Some(2));
        assert!(handle.subtitle_tracks().iter().find(|t| t.id == 2).unwrap().is_selected);
        assert!(!handle.subtitle_tracks().iter().find(|t| t.id == 1).unwrap().is_selected);

        // Verify cue extraction for ALL tracks using their respective ff_index or external_filename
        for (i, t) in sub_tracks.iter().enumerate() {
            if let Some(ext_file) = &t.external_filename {
                let content = std::fs::read_to_string(ext_file).expect("read external srt");
                let cues = wazoo_media::parse_subtitles(&content);
                assert!(!cues.is_empty(), "Track {} ({}) external cues empty", i, ext_file);
                println!("Extracted {} cues for track {} from external file {}", cues.len(), i, ext_file);
            } else {
                let cues = wazoo_media::load_subtitles_for_stream_sync(sample, t.ff_index, i);
                assert!(!cues.is_empty(), "Track {} (ff_index: {:?}) cues empty", i, t.ff_index);
                println!("Extracted {} cues for track {} (ff_index: {:?})", cues.len(), i, t.ff_index);
            }
        }
    }

    let sample16 = "/mnt/bob/anime/Mushoku Tensei/Season 2/S02E16-Norn and Aisha.mkv";
    if Path::new(sample16).exists() {
        let mut handle = VideoHandle::new(13, sample16, "Norn and Aisha").expect("create VideoHandle");
        handle.set_muted(true);

        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(100));
            let _ = handle.update_frame();
            if handle.subtitle_tracks().len() >= 3 {
                break;
            }
        }

        let sub_tracks = handle.subtitle_tracks();
        println!("S02E16: Found {} subtitle tracks", sub_tracks.len());
        for (i, t) in sub_tracks.iter().enumerate() {
            println!("  S02E16 [{}] id={} selected={} ff_index={:?} ext={:?} label='{}'",
                i, t.id, t.is_selected, t.ff_index, t.external_filename, wazoo_media::format_subtitle_track_label(t, i)
            );

            // Test extracting cues for every single track in S02E16
            if let Some(ext_file) = &t.external_filename {
                let content = std::fs::read_to_string(ext_file).expect("read external srt");
                let cues = wazoo_media::parse_subtitles(&content);
                assert!(!cues.is_empty(), "S02E16 Track {} ({}) external cues empty", i, ext_file);
                println!("  -> Extracted {} cues from external file {}", cues.len(), ext_file);
            } else {
                let cues = wazoo_media::load_subtitles_for_stream_sync(sample16, t.ff_index, i);
                assert!(!cues.is_empty(), "S02E16 Track {} (ff_index: {:?}) cues empty", i, t.ff_index);
                println!("  -> Extracted {} cues via ffmpeg (ff_index: {:?})", cues.len(), t.ff_index);
            }
        }
    }
}
