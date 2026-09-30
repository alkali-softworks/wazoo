use wazoo_media::subtitles::{clean_subtitle_text, parse_ass, parse_srt_or_vtt, parse_timestamp};

#[test]
fn test_parse_timestamp() {
    assert_eq!(parse_timestamp("00:00:14,760"), Some(14.76));
    assert_eq!(parse_timestamp("01:23.450"), Some(83.45));
    assert_eq!(parse_timestamp("1:02:03.50"), Some(3723.5));
    assert_eq!(parse_timestamp("invalid"), None);
    assert_eq!(parse_timestamp("NaN:NaN:NaN"), None);
    assert_eq!(parse_timestamp("inf"), None);
    assert_eq!(parse_timestamp("-01:00"), None);
}

#[test]
fn test_clean_subtitle_text() {
    let input = r#"<font face="Chalk" size="30">{\an8}Wow, it's windy.\NReally windy!</font>"#;
    assert_eq!(clean_subtitle_text(input), "Wow, it's windy. Really windy!");
}

#[test]
fn test_parse_srt() {
    let srt = r#"
1
00:00:10,000 --> 00:00:14,000
First subtitle line.

2
00:00:15,500 --> 00:00:19,000
Second subtitle line with <i>italics</i>.
"#;
    let cues = parse_srt_or_vtt(srt);
    assert_eq!(cues.len(), 2);
    assert_eq!(cues[0].start_secs, 10.0);
    assert_eq!(cues[0].end_secs, 14.0);
    assert_eq!(cues[0].text, "First subtitle line.");
    assert_eq!(cues[1].start_secs, 15.5);
    assert_eq!(cues[1].end_secs, 19.0);
    assert_eq!(cues[1].text, "Second subtitle line with italics.");
}

#[test]
fn test_parse_ass() {
    let ass = r#"
[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:14.76,0:00:18.49,Default,,0,0,0,,{\an8}Wow, it's windy.
Dialogue: 0,0:00:20.00,0:00:22.50,Default,,0,0,0,,Second line.
"#;
    let cues = parse_ass(ass);
    assert_eq!(cues.len(), 2);
    assert_eq!(cues[0].start_secs, 14.76);
    assert_eq!(cues[0].end_secs, 18.49);
    assert_eq!(cues[0].text, "Wow, it's windy.");
    assert_eq!(cues[1].start_secs, 20.0);
    assert_eq!(cues[1].end_secs, 22.5);
    assert_eq!(cues[1].text, "Second line.");
}

#[tokio::test]
async fn test_load_subtitles_for_track_details_external_file() {
    let dir = std::env::temp_dir().join(format!("wazoo_sub_test_{}", rand::random::<u32>()));
    std::fs::create_dir_all(&dir).unwrap();
    let srt_path = dir.join("test_track.srt");
    std::fs::write(
        &srt_path,
        "1\n00:00:01,000 --> 00:00:04,000\nAsync subtitle loaded.\n",
    )
    .unwrap();

    let cues = wazoo_media::load_subtitles_for_track_details(
        "dummy_video.mp4".to_string(),
        Some(srt_path.to_string_lossy().to_string()),
        None,
        0,
    )
    .await;

    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].start_secs, 1.0);
    assert_eq!(cues[0].end_secs, 4.0);
    assert_eq!(cues[0].text, "Async subtitle loaded.");

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_run_ffmpeg_subtitle_extract_async_nonexistent_file() {
    let cues = wazoo_media::run_ffmpeg_subtitle_extract_async(
        "/nonexistent/path/to/video_file_12345.mkv",
        "0:s:0",
    )
    .await;

    // Gracefully returns None without panicking or blocking
    assert!(cues.is_none());
}

#[tokio::test]
async fn test_oversized_subtitle_file_rejected() {
    let dir = std::env::temp_dir().join(format!("wazoo_sub_oversize_{}", rand::random::<u32>()));
    std::fs::create_dir_all(&dir).unwrap();
    let srt_path = dir.join("oversized.srt");

    // Create a sparse file larger than MAX_SUBTITLE_FILE_BYTES (10 MB + 1 byte)
    let file = std::fs::File::create(&srt_path).unwrap();
    file.set_len(10 * 1024 * 1024 + 1).unwrap();

    let cues = wazoo_media::load_subtitles_for_track_details(
        "dummy_video.mp4".to_string(),
        Some(srt_path.to_string_lossy().to_string()),
        None,
        0,
    )
    .await;

    // Must be rejected without attempting to read into memory
    assert!(cues.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_invalid_map_arg_rejected() {
    // Malicious map args with command injection, path traversal, or option flags
    let bad_args = [
        "0:s:0; rm -rf /",
        "-nostdin",
        "0:s:0 | echo hacked",
        "0:s:0\n-v error",
        "",
    ];

    for bad in &bad_args {
        let res = wazoo_media::run_ffmpeg_subtitle_extract_async("dummy.mp4", bad).await;
        assert!(res.is_none(), "Invalid map_arg '{}' should be rejected", bad);
    }
}

#[test]
fn test_max_cues_limit() {
    // Generate an SRT stream exceeding MAX_SUBTITLE_CUES (50,000)
    use std::fmt::Write;
    let mut huge_srt = String::new();
    for i in 0..50_050 {
        let _ = writeln!(
            &mut huge_srt,
            "{}\n00:00:01,000 --> 00:00:02,000\nLine {}\n",
            i + 1,
            i
        );
    }

    let cues = parse_srt_or_vtt(&huge_srt);
    assert_eq!(
        cues.len(),
        wazoo_media::subtitles::MAX_SUBTITLE_CUES,
        "Cue parsing should be bounded by MAX_SUBTITLE_CUES"
    );
}
