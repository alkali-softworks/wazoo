use std::ffi::CString;
use wazoo_media::{
    AudioTrack, BufferConfig, SubtitleTrack, VideoHandle, build_alang_string, build_slang_string,
    build_slang_string_with_fallback, find_matching_audio_track, find_matching_subtitle_track,
    format_subtitle_track_label, get_subtitle_track_preference_string,
    get_track_preference_string, is_forced_track, is_signs_or_songs_track, mpv_ffi,
    select_best_subtitle_track, select_best_subtitle_track_with_fallback,
    subtitle_track_matches_preference, track_matches_preference,
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

#[test]
fn test_video_equalizer_filter_generation() {
    // Default zero values should produce an empty filter string (bypassing vf overhead)
    assert_eq!(
        wazoo_media::VideoHandle::build_eq_filter_string(0.0, 0.0, 0.0, 0.0),
        ""
    );

    // Non-zero values generate ffmpeg eq filter parameters
    let filter = wazoo_media::VideoHandle::build_eq_filter_string(50.0, 50.0, 25.0, -50.0);
    assert!(filter.starts_with("eq="));
    assert!(filter.contains("gamma=1.750"));
    assert!(filter.contains("contrast=1.750"));
    assert!(filter.contains("brightness=0.200"));
    assert!(filter.contains("saturation=0.500"));

    // Extreme values
    let filter_max = wazoo_media::VideoHandle::build_eq_filter_string(100.0, 100.0, 100.0, 100.0);
    assert!(filter_max.contains("gamma=2.500"));
    assert!(filter_max.contains("contrast=2.500"));
    assert!(filter_max.contains("brightness=0.800"));
    assert!(filter_max.contains("saturation=2.500"));
}

#[test]
fn test_player_mark_in_out() {
    use std::time::Duration;
    use wazoo_media::PlayerState;

    let mut state = PlayerState::new(1, "test.mp4".to_string(), "test".to_string());
    assert_eq!(state.mark_in, None);
    assert_eq!(state.mark_out, None);

    // Set mark in
    state.mark_in = Some(Duration::from_secs(5));
    assert_eq!(state.mark_in, Some(Duration::from_secs(5)));
    assert_eq!(state.mark_out, None);

    // Set mark out
    state.mark_out = Some(Duration::from_secs(10));
    assert_eq!(state.mark_in, Some(Duration::from_secs(5)));
    assert_eq!(state.mark_out, Some(Duration::from_secs(10)));

    // Clear mark in
    state.mark_in = None;
    assert_eq!(state.mark_in, None);
    assert_eq!(state.mark_out, Some(Duration::from_secs(10)));

    // Clear all
    state.mark_out = None;
    assert_eq!(state.mark_in, None);
    assert_eq!(state.mark_out, None);
}

#[test]
fn test_matches_alias_token_avoids_substring_false_positives() {
    use wazoo_media::matches_alias_token;

    // Short alias "en" must NOT match as substring in arbitrary words
    assert!(!matches_alias_token("Clean", "en"));
    assert!(!matches_alias_token("Opening Theme", "en"));
    assert!(!matches_alias_token("French Audio", "en"));
    assert!(!matches_alias_token("Scene Audio", "en"));

    // Short alias "es" must NOT match in words like Remastered
    assert!(!matches_alias_token("Remastered Audio", "es"));
    assert!(!matches_alias_token("Effects Track", "es"));

    // But should match as discrete tokens
    assert!(matches_alias_token("Audio [EN]", "en"));
    assert!(matches_alias_token("Track (en)", "en"));
    assert!(matches_alias_token("Dub - en - 2.0", "en"));
    assert!(matches_alias_token("Audio [ES]", "es"));

    // Longer aliases work with normal substring matching
    assert!(matches_alias_token("English Dub 5.1", "english"));
    assert!(matches_alias_token("Stereo [eng]", "eng"));
    assert!(matches_alias_token("Japanese Stereo", "japanese"));
    assert!(matches_alias_token("Japanese Audio [jpn]", "jpn"));
    assert!(matches_alias_token("Spanish Dubbed", "spanish"));
}

#[test]
fn test_multitrack_preference_matching() {
    use wazoo_media::{AudioTrack, find_matching_audio_track};

    // Tracks matching Slayers MKV episodes (Stream 1: eng, Stream 2: jpn/Stereo, Stream 3: spa)
    let tracks = vec![
        AudioTrack {
            id: 1,
            title: None,
            lang: Some("eng".to_string()),
            codec: Some("vorbis".to_string()),
            is_selected: true,
        },
        AudioTrack {
            id: 2,
            title: Some("Stereo".to_string()),
            lang: Some("jpn".to_string()),
            codec: Some("vorbis".to_string()),
            is_selected: false,
        },
        AudioTrack {
            id: 3,
            title: None,
            lang: Some("spa".to_string()),
            codec: Some("vorbis".to_string()),
            is_selected: false,
        },
    ];

    // English preference -> Track 1
    assert_eq!(find_matching_audio_track(&tracks, "English"), Some(1));
    assert_eq!(find_matching_audio_track(&tracks, "eng"), Some(1));
    assert_eq!(find_matching_audio_track(&tracks, "en"), Some(1));

    // Japanese preference -> Track 2
    assert_eq!(find_matching_audio_track(&tracks, "Japanese"), Some(2));
    assert_eq!(find_matching_audio_track(&tracks, "jpn"), Some(2));
    assert_eq!(find_matching_audio_track(&tracks, "ja"), Some(2));

    // Spanish preference -> Track 3
    assert_eq!(find_matching_audio_track(&tracks, "Spanish"), Some(3));
    assert_eq!(find_matching_audio_track(&tracks, "spa"), Some(3));
    assert_eq!(find_matching_audio_track(&tracks, "es"), Some(3));

    // Non-existent preference -> None
    assert_eq!(find_matching_audio_track(&tracks, "German"), None);
}

#[test]
fn test_format_audio_track_label_prefers_track_id() {
    use wazoo_media::{AudioTrack, format_audio_track_label};

    let track_untagged = AudioTrack {
        id: 3,
        title: None,
        lang: None,
        codec: None,
        is_selected: false,
    };

    // Even when passing index 0, should format using track.id = 3 -> "Track 3"
    assert_eq!(format_audio_track_label(&track_untagged, 0), "Track 3");
}

#[test]
fn test_audio_track_preference_updates_and_retention() {
    use wazoo_media::{AudioTrack, get_track_preference_string};

    let track_spanish = AudioTrack {
        id: 3,
        title: None,
        lang: Some("spa".to_string()),
        codec: Some("vorbis".to_string()),
        is_selected: true,
    };
    assert_eq!(get_track_preference_string(&track_spanish), "Spanish");

    let track_english = AudioTrack {
        id: 1,
        title: None,
        lang: Some("eng".to_string()),
        codec: Some("vorbis".to_string()),
        is_selected: false,
    };
    assert_eq!(get_track_preference_string(&track_english), "English");

    let track_japanese = AudioTrack {
        id: 2,
        title: Some("Stereo".to_string()),
        lang: Some("jpn".to_string()),
        codec: Some("vorbis".to_string()),
        is_selected: false,
    };
    assert_eq!(get_track_preference_string(&track_japanese), "Japanese");
}

#[test]
fn test_ranma_subtitle_selection_defaults_to_english_not_arabic() {
    let path = "/mnt/bob/anime/Ranma/Season 1/Ranma 1_2 - 122.mkv";
    if !std::path::Path::new(path).exists() {
        return;
    }
    let mut handle = wazoo_media::VideoHandle::new(1, path, "Ranma").expect("handle creation");
    let start = std::time::Instant::now();
    let mut selected_id = None;
    while start.elapsed() < std::time::Duration::from_millis(3000) {
        handle.update_frame();
        if let Some(id) = handle.current_subtitle_track_id() {
            selected_id = Some(id);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let cur_id = selected_id.expect("Subtitle track should be selected");
    // Track 1 is Arabic ('ara'). Must NEVER default to Track 1.
    assert_ne!(cur_id, 1, "Must not default to Arabic subtitle track (id 1)");
    // Should be Track 5 (English default) or external .srt
    let active_track = handle
        .subtitle_tracks()
        .into_iter()
        .find(|t| t.id == cur_id)
        .expect("Selected track must exist");
    assert!(
        active_track.lang.as_deref() == Some("eng")
            || active_track.title.as_deref().map_or(false, |t| t.ends_with(".srt")),
        "Default subtitle track must be English, found: id={}, lang={:?}, title={:?}",
        active_track.id,
        active_track.lang,
        active_track.title
    );
}

#[test]
fn test_select_best_subtitle_track_ranma_scenario() {
    let tracks = vec![
        SubtitleTrack {
            id: 1,
            title: Some("[ara] كرشرول".to_string()),
            lang: Some("ara".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: false,
            is_forced: false,
            external_filename: None,
            ff_index: Some(2),
        },
        SubtitleTrack {
            id: 2,
            title: Some("[cat] Animelliure [Forçat]".to_string()),
            lang: Some("cat".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: false,
            is_forced: true,
            external_filename: None,
            ff_index: Some(3),
        },
        SubtitleTrack {
            id: 3,
            title: Some("[chi] Cornflower Studio [GB]".to_string()),
            lang: Some("chi".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: false,
            is_forced: false,
            external_filename: None,
            ff_index: Some(4),
        },
        SubtitleTrack {
            id: 5,
            title: Some("[eng] Doki/grimf/der richter/Refha".to_string()),
            lang: Some("eng".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: true,
            is_forced: false,
            external_filename: None,
            ff_index: Some(6),
        },
        SubtitleTrack {
            id: 9,
            title: Some("[spa] Animelliure".to_string()),
            lang: Some("spa".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: false,
            is_forced: false,
            external_filename: None,
            ff_index: Some(10),
        },
        SubtitleTrack {
            id: 10,
            title: Some("[spa] Animelliure [Forzado]".to_string()),
            lang: Some("spa".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: false,
            is_forced: true,
            external_filename: None,
            ff_index: Some(11),
        },
    ];

    // Default preference (English) selects English Track 5
    assert_eq!(find_matching_subtitle_track(&tracks, "English"), Some(5));
    assert_eq!(find_matching_subtitle_track(&tracks, "eng"), Some(5));
    assert_eq!(select_best_subtitle_track(&tracks, None), Some(5));
    assert_eq!(select_best_subtitle_track(&tracks, Some("English")), Some(5));
    assert_eq!(select_best_subtitle_track(&tracks, Some("eng")), Some(5));

    // Spanish preference selects full Spanish track 9 over forced track 10
    assert_eq!(find_matching_subtitle_track(&tracks, "Spanish"), Some(9));
    assert_eq!(find_matching_subtitle_track(&tracks, "spa"), Some(9));
    assert_eq!(select_best_subtitle_track(&tracks, Some("Spanish")), Some(9));
    assert_eq!(select_best_subtitle_track(&tracks, Some("spa")), Some(9));

    // Preference for unmatched language falls back to English track 5
    assert_eq!(select_best_subtitle_track(&tracks, Some("German")), Some(5));
}

#[test]
fn test_select_best_subtitle_track_signs_and_songs_deprioritized() {
    let tracks = vec![
        SubtitleTrack {
            id: 1,
            title: Some("English Signs & Songs".to_string()),
            lang: Some("eng".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: false,
            is_forced: false,
            external_filename: None,
            ff_index: Some(2),
        },
        SubtitleTrack {
            id: 2,
            title: Some("English Full Dialogue".to_string()),
            lang: Some("eng".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: false,
            is_forced: false,
            external_filename: None,
            ff_index: Some(3),
        },
    ];

    assert!(is_signs_or_songs_track(&tracks[0]));
    assert!(!is_signs_or_songs_track(&tracks[1]));
    assert_eq!(select_best_subtitle_track(&tracks, Some("English")), Some(2));
}

#[test]
fn test_subtitle_track_preference_string_and_aliases() {
    let track_eng = SubtitleTrack {
        id: 5,
        title: Some("[eng] Doki".to_string()),
        lang: Some("eng".to_string()),
        codec: Some("ass".to_string()),
        is_selected: false,
        is_default: true,
        is_forced: false,
        external_filename: None,
        ff_index: Some(6),
    };
    assert_eq!(get_subtitle_track_preference_string(&track_eng), "English");
    assert!(subtitle_track_matches_preference(&track_eng, "English"));
    assert!(subtitle_track_matches_preference(&track_eng, "eng"));
    assert!(subtitle_track_matches_preference(&track_eng, "en"));
    assert!(!subtitle_track_matches_preference(&track_eng, "Arabic"));

    assert_eq!(build_slang_string("English"), "en,eng,english");
    assert_eq!(build_slang_string("Spanish"), "es,spa,spanish");
}

#[test]
fn test_is_forced_track_detection() {
    let forced_by_flag = SubtitleTrack {
        id: 1,
        title: None,
        lang: Some("eng".to_string()),
        codec: None,
        is_selected: false,
        is_default: false,
        is_forced: true,
        external_filename: None,
        ff_index: None,
    };
    assert!(is_forced_track(&forced_by_flag));

    let forced_by_title_es = SubtitleTrack {
        id: 2,
        title: Some("[spa] Animelliure [Forzado]".to_string()),
        lang: Some("spa".to_string()),
        codec: None,
        is_selected: false,
        is_default: false,
        is_forced: false,
        external_filename: None,
        ff_index: None,
    };
    assert!(is_forced_track(&forced_by_title_es));

    let forced_by_title_cat = SubtitleTrack {
        id: 3,
        title: Some("[cat] Animelliure [Forçat]".to_string()),
        lang: Some("cat".to_string()),
        codec: None,
        is_selected: false,
        is_default: false,
        is_forced: false,
        external_filename: None,
        ff_index: None,
    };
    assert!(is_forced_track(&forced_by_title_cat));

    let normal_track = SubtitleTrack {
        id: 4,
        title: Some("[eng] Full dialogue".to_string()),
        lang: Some("eng".to_string()),
        codec: None,
        is_selected: false,
        is_default: false,
        is_forced: false,
        external_filename: None,
        ff_index: None,
    };
    assert!(!is_forced_track(&normal_track));
}

#[test]
fn test_select_best_subtitle_track_with_i18n_fallback() {
    let tracks = vec![
        SubtitleTrack {
            id: 1,
            title: Some("[ara] كرشرول".to_string()),
            lang: Some("ara".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: false,
            is_forced: false,
            external_filename: None,
            ff_index: Some(2),
        },
        SubtitleTrack {
            id: 3,
            title: Some("[chi] Cornflower Studio [GB]".to_string()),
            lang: Some("chi".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: false,
            is_forced: false,
            external_filename: None,
            ff_index: Some(4),
        },
        SubtitleTrack {
            id: 5,
            title: Some("[eng] Doki/grimf/der richter/Refha".to_string()),
            lang: Some("eng".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: true,
            is_forced: false,
            external_filename: None,
            ff_index: Some(6),
        },
        SubtitleTrack {
            id: 9,
            title: Some("[spa] Animelliure".to_string()),
            lang: Some("spa".to_string()),
            codec: Some("ass".to_string()),
            is_selected: false,
            is_default: false,
            is_forced: false,
            external_filename: None,
            ff_index: Some(10),
        },
    ];

    // If preferred_subtitle_language is not known (None), use the "language" i18n setting
    assert_eq!(
        select_best_subtitle_track_with_fallback(&tracks, None, Some("es")),
        Some(9)
    );
    assert_eq!(
        select_best_subtitle_track_with_fallback(&tracks, None, Some("Spanish")),
        Some(9)
    );
    assert_eq!(
        select_best_subtitle_track_with_fallback(&tracks, None, Some("chi")),
        Some(3)
    );
    assert_eq!(
        select_best_subtitle_track_with_fallback(&tracks, None, Some("en")),
        Some(5)
    );

    // If preferred is explicitly set (known), it takes precedence over i18n setting
    assert_eq!(
        select_best_subtitle_track_with_fallback(&tracks, Some("English"), Some("es")),
        Some(5)
    );
    assert_eq!(
        select_best_subtitle_track_with_fallback(&tracks, Some("Spanish"), Some("en")),
        Some(9)
    );

    // If preferred is known but not found in the file, it falls back to the i18n setting
    assert_eq!(
        select_best_subtitle_track_with_fallback(&tracks, Some("Japanese"), Some("es")),
        Some(9)
    );

    // If neither preferred nor i18n is found, falls back to English (default track 5)
    assert_eq!(
        select_best_subtitle_track_with_fallback(&tracks, Some("German"), Some("French")),
        Some(5)
    );
}

#[test]
fn test_build_slang_string_with_fallback() {
    // When preferred is not known, uses i18n language setting
    let slang_es = build_slang_string_with_fallback(None, Some("es"));
    assert!(slang_es.starts_with("es,spa,spanish"));
    assert!(slang_es.contains("en,eng,english"));

    // When preferred is known, preferred comes first, then i18n, then English
    let slang_ja_es = build_slang_string_with_fallback(Some("Japanese"), Some("es"));
    assert!(slang_ja_es.starts_with("ja,jpn,jp,japanese"));
    assert!(slang_ja_es.contains("es,spa,spanish"));
    assert!(slang_ja_es.contains("en,eng,english"));
}

#[test]
fn test_buffer_config_effective_subtitle_language() {
    let mut config = BufferConfig::default();
    assert_eq!(config.preferred_subtitle_language, None);
    assert_eq!(config.i18n_language.as_deref(), Some("en"));
    assert_eq!(config.effective_subtitle_language(), "en");

    config.i18n_language = Some("es".to_string());
    assert_eq!(config.effective_subtitle_language(), "es");

    config.preferred_subtitle_language = Some("French".to_string());
    assert_eq!(config.effective_subtitle_language(), "French");

    config.preferred_subtitle_language = None;
    assert_eq!(config.effective_subtitle_language(), "es");
}

#[test]
fn test_ranma_subtitle_selection_uses_i18n_when_preferred_not_known() {
    let path = "/mnt/bob/anime/Ranma/Season 1/Ranma 1_2 - 122.mkv";
    if !std::path::Path::new(path).exists() {
        return;
    }

    // When preferred_subtitle_language is not known and i18n is "es", selects Spanish (Track 9)
    let mut config_es = BufferConfig::default();
    config_es.preferred_subtitle_language = None;
    config_es.i18n_language = Some("es".to_string());

    let mut handle_es = VideoHandle::with_buffering(1, path, "Ranma", config_es).expect("handle");
    let start = std::time::Instant::now();
    let mut selected_id = None;
    while start.elapsed() < std::time::Duration::from_millis(3000) {
        handle_es.update_frame();
        if let Some(id) = handle_es.current_subtitle_track_id() {
            selected_id = Some(id);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert_eq!(selected_id, Some(9), "Must select Spanish track (id 9) matching i18n language setting");
}

