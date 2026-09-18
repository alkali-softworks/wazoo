use std::ffi::CString;
use wazoo_media::{
    build_alang_string, find_matching_audio_track, format_subtitle_track_label,
    get_track_preference_string, mpv_ffi, track_matches_preference, AudioTrack, SubtitleTrack,
};

#[test]
fn test_find_matching_audio_track() {
    let tracks = vec![
        AudioTrack {
            id: 1,
            title: Some("English Dub / AAC LC".to_string()),
            lang: Some("eng".to_string()),
            codec: Some("aac".to_string()),
            is_selected: true,
        },
        AudioTrack {
            id: 2,
            title: None,
            lang: Some("jpn".to_string()),
            codec: Some("aac".to_string()),
            is_selected: false,
        },
    ];

    // Matching by language code or display name
    assert_eq!(find_matching_audio_track(&tracks, "Japanese"), Some(2));
    assert_eq!(find_matching_audio_track(&tracks, "jpn"), Some(2));
    assert_eq!(find_matching_audio_track(&tracks, "ja"), Some(2));
    assert_eq!(find_matching_audio_track(&tracks, "English"), Some(1));
    assert_eq!(find_matching_audio_track(&tracks, "eng"), Some(1));

    // Matching by title substring
    assert_eq!(find_matching_audio_track(&tracks, "dub"), Some(1));

    // Unknown language returns None
    assert_eq!(find_matching_audio_track(&tracks, "German"), None);
    assert_eq!(find_matching_audio_track(&tracks, ""), None);
}

#[test]
fn test_get_track_preference_string() {
    let track_jpn = AudioTrack {
        id: 2,
        title: None,
        lang: Some("jpn".to_string()),
        codec: None,
        is_selected: false,
    };
    assert_eq!(get_track_preference_string(&track_jpn), "Japanese");

    let track_eng = AudioTrack {
        id: 1,
        title: Some("English Dub / AAC LC".to_string()),
        lang: Some("eng".to_string()),
        codec: None,
        is_selected: true,
    };
    assert_eq!(get_track_preference_string(&track_eng), "English");

    let track_commentary = AudioTrack {
        id: 3,
        title: Some("Director's Commentary".to_string()),
        lang: None,
        codec: None,
        is_selected: false,
    };
    assert_eq!(
        get_track_preference_string(&track_commentary),
        "Director's Commentary"
    );
}

#[test]
fn test_build_alang_string() {
    assert_eq!(build_alang_string("Japanese"), "ja,jpn,jp,japanese");
    assert_eq!(build_alang_string("English"), "en,eng,english");
}

#[test]
fn test_format_subtitle_track_label() {
    let t1 = SubtitleTrack {
        id: 1,
        title: Some("Full Subtitles / English / ASS / MTBB".to_string()),
        lang: Some("enm".to_string()),
        codec: Some("ass".to_string()),
        is_selected: true,
        ..Default::default()
    };
    assert_eq!(
        format_subtitle_track_label(&t1, 0),
        "English (Full Subtitles)"
    );

    let t2 = SubtitleTrack {
        id: 2,
        title: Some("Full Subtitles / English / ASS / MTBB / Honorofics".to_string()),
        lang: Some("enm".to_string()),
        codec: Some("ass".to_string()),
        is_selected: false,
        ..Default::default()
    };
    assert_eq!(
        format_subtitle_track_label(&t2, 1),
        "English (Full Subtitles - Honorifics)"
    );

    let t3 = SubtitleTrack {
        id: 3,
        title: Some("Signs and Songs".to_string()),
        lang: Some("eng".to_string()),
        codec: Some("ass".to_string()),
        is_selected: false,
        ..Default::default()
    };
    assert_eq!(
        format_subtitle_track_label(&t3, 2),
        "English (Signs and Songs)"
    );

    let t4 = SubtitleTrack {
        id: 4,
        title: None,
        lang: Some("ja".to_string()),
        codec: None,
        is_selected: false,
        ..Default::default()
    };
    assert_eq!(format_subtitle_track_label(&t4, 3), "Japanese");

    let t5 = SubtitleTrack {
        id: 5,
        title: None,
        lang: None,
        codec: None,
        is_selected: false,
        ..Default::default()
    };
    assert_eq!(format_subtitle_track_label(&t5, 4), "Track 5");
}

#[test]
fn test_mpv_start_option() {
    unsafe {
        let mpv = mpv_ffi::mpv_create();
        assert!(!mpv.is_null());
        let k = CString::new("start").unwrap();
        let v = CString::new("25%").unwrap();
        let res = mpv_ffi::mpv_set_option_string(mpv, k.as_ptr(), v.as_ptr());
        assert_eq!(res, 0, "mpv must accept percentage for start option");
        mpv_ffi::mpv_terminate_destroy(mpv);
    }
}

#[test]
fn test_track_matches_preference_same_language_commentary() {
    let main_track = AudioTrack {
        id: 1,
        title: Some("Surround".to_string()),
        lang: Some("eng".to_string()),
        codec: Some("aac".to_string()),
        is_selected: true,
    };
    let commentary_1 = AudioTrack {
        id: 2,
        title: Some("Commentary by film historian David Kalat".to_string()),
        lang: Some("eng".to_string()),
        codec: Some("aac".to_string()),
        is_selected: false,
    };
    let commentary_2 = AudioTrack {
        id: 3,
        title: Some("Commentary by film critic Brad Stevens".to_string()),
        lang: Some("eng".to_string()),
        codec: Some("aac".to_string()),
        is_selected: false,
    };

    // Both main track and commentary tracks match English preference
    assert!(track_matches_preference(&main_track, "English"));
    assert!(track_matches_preference(&commentary_1, "English"));
    assert!(track_matches_preference(&commentary_2, "English"));

    // Does not match other languages
    assert!(!track_matches_preference(&commentary_1, "Japanese"));
    assert!(!track_matches_preference(&commentary_1, "Spanish"));
}

