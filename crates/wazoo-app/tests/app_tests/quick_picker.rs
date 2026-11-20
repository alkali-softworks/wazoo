use super::common::{TempTestDir, press_char};
use wazoo_app::app::new_test_app;
use wazoo_app::message::Message;
use wazoo_core::{MediaMetadata, VideoRecord};

#[test]
fn test_quick_picker_videos_folder_isolation() {
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("test_qp_iso");
    let show_a = temp_dir.path().join("Show A");
    let show_b = temp_dir.path().join("Show B");
    let _ = std::fs::create_dir_all(&show_a);
    let _ = std::fs::create_dir_all(&show_b);

    let a1 = show_a.join("s01e01.mp4");
    let a2 = show_a.join("s01e02.mp4");
    let a3 = show_a.join("s01e03.mp4");
    let b1 = show_b.join("ep01.mp4");
    let b2 = show_b.join("ep02.mp4");

    let _ = std::fs::File::create(&a1);
    let _ = std::fs::File::create(&a2);
    let _ = std::fs::File::create(&a3);
    let _ = std::fs::File::create(&b1);
    let _ = std::fs::File::create(&b2);

    let p_a1 = a1.to_string_lossy().to_string();
    let p_a2 = a2.to_string_lossy().to_string();
    let p_a3 = a3.to_string_lossy().to_string();
    let p_b1 = b1.to_string_lossy().to_string();
    let p_b2 = b2.to_string_lossy().to_string();

    // Populate available videos across different folders
    app.available_videos = vec![
        VideoRecord::new(1, "Show A - S01E01", &p_a1),
        VideoRecord::new(2, "Show A - S01E02", &p_a2),
        VideoRecord::new(3, "Show A - S01E03", &p_a3),
        VideoRecord::new(4, "Show B - EP01", &p_b1),
        VideoRecord::new(5, "Show B - EP02", &p_b2),
    ];
    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);

    // Case 1: When player has no path
    app.players[0].state.path.clear();
    let (folder, candidates) = app.quick_picker_videos();
    assert_eq!(folder, None);
    assert!(candidates.is_empty());

    // Case 2: Play Show A - S01E02
    app.players[0].state.path = p_a2.clone();
    let (folder, candidates) = app.quick_picker_videos();
    assert_eq!(folder.as_deref(), Some("Show A"));
    // Exactly 2 OTHER videos from Show A
    assert_eq!(candidates.len(), 2);
    let paths: Vec<&str> = candidates.iter().map(|(p, _)| p.as_str()).collect();
    assert!(paths.contains(&p_a1.as_str()));
    assert!(paths.contains(&p_a3.as_str()));
    // Crucially: currently playing video is NOT in the other videos list
    assert!(!paths.contains(&p_a2.as_str()));
    // Videos from Show B are NOT in the list
    assert!(!paths.contains(&p_b1.as_str()));

    // Case 3: Play Show B - EP01
    app.players[0].state.path = p_b1.clone();
    let (folder, candidates) = app.quick_picker_videos();
    assert_eq!(folder.as_deref(), Some("Show B"));
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].0, p_b2);
}

#[test]
fn test_quick_picker_modal_k_key_escape_and_actions() {
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("test_qp_actions");
    let show_dir = temp_dir.path().join("My Show");
    let _ = std::fs::create_dir_all(&show_dir);

    let f1 = show_dir.join("Episode 1.mp4");
    let f2 = show_dir.join("Episode 2.mp4");
    let f3 = show_dir.join("Episode 3.mp4");

    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);
    let _ = std::fs::File::create(&f3);

    let p1 = f1.to_string_lossy().to_string();
    let p2 = f2.to_string_lossy().to_string();
    let p3 = f3.to_string_lossy().to_string();

    app.available_videos = vec![
        VideoRecord::new(1, "Episode 1", &p1),
        VideoRecord::new(2, "Episode 2", &p2),
        VideoRecord::new(3, "Episode 3", &p3),
    ];
    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);
    app.players[0].state.path = p1.clone();

    assert!(!app.modals.quick_picker);
    assert!(!app.modals.is_any_open());

    // 1. Press 'k' key to open quick picker modal
    press_char(&mut app, "k");
    assert!(app.modals.quick_picker);
    assert!(app.modals.is_any_open());
    assert_eq!(app.modals.quick_picker_search, "");

    // 2. Typing in search filter filters the videos
    let _ = app.update(Message::QuickPickerSearchChanged("3".to_string()));
    assert_eq!(app.modals.quick_picker_search, "3");

    // 3. Submitting via Enter plays the first matching file and closes modal
    let _ = app.update(Message::QuickPickerSubmit);
    assert!(!app.modals.quick_picker);
    assert_eq!(app.players[0].state.path, p3);

    // 4. Press 'k' key again opens modal
    press_char(&mut app, "k");
    assert!(app.modals.quick_picker);

    // 5. Escape closes the quick picker modal without modifying playback
    let _ = app.update(Message::EscapePressed);
    assert!(!app.modals.quick_picker);
    assert_eq!(app.players[0].state.path, p3);

    // 6. Direct CloseQuickPickerModal message closes it
    let _ = app.update(Message::OpenQuickPickerModal);
    assert!(app.modals.quick_picker);
    let _ = app.update(Message::CloseQuickPickerModal);
    assert!(!app.modals.quick_picker);

    // 7. ClearQuickPickerSearch clears the filter
    let _ = app.update(Message::OpenQuickPickerModal);
    let _ = app.update(Message::QuickPickerSearchChanged("filter_xyz".to_string()));
    let _ = app.update(Message::ClearQuickPickerSearch);
    assert_eq!(app.modals.quick_picker_search, "");

    // 8. PlayQuickPickerFile plays selected file and closes modal
    let _ = app.update(Message::PlayQuickPickerFile(p2.clone()));
    assert!(!app.modals.quick_picker);
    assert_eq!(app.players[0].state.path, p2);

    // 9. When no other videos exist in folder, quick picker does not open and displays toast
    app.available_videos = vec![VideoRecord::new(2, "Episode 2", &p2)];
    let _ = std::fs::remove_file(&f1);
    let _ = std::fs::remove_file(&f3);
    app.overlay.clear_toast();
    let _ = app.update(Message::OpenQuickPickerModal);
    assert!(!app.modals.quick_picker);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some(app.t("wazoo.no_files_found").as_str())
    );

    // 10. When no video is playing, quick picker does not open and displays toast
    app.players[0].state.path.clear();
    app.overlay.clear_toast();
    press_char(&mut app, "k");
    assert!(!app.modals.quick_picker);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some(app.t("wazoo.no_files_found").as_str())
    );
}

#[test]
fn test_quick_picker_modal_render() {
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("test_qp_render");
    let show_dir = temp_dir.path().join("Anime");
    let _ = std::fs::create_dir_all(&show_dir);
    let ep1 = show_dir.join("ep01.mp4");
    let ep2 = show_dir.join("ep02.mp4");
    let _ = std::fs::File::create(&ep1);
    let _ = std::fs::File::create(&ep2);

    let p1 = ep1.to_string_lossy().to_string();
    let p2 = ep2.to_string_lossy().to_string();

    app.available_videos = vec![
        VideoRecord::new(1, "Anime - EP 01", &p1),
        VideoRecord::new(2, "Anime - EP 02", &p2),
    ];
    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);

    // Render with video playing and other video in folder
    app.players[0].state.path = p1.clone();
    app.modals.quick_picker = true;
    {
        let _el = app.view_quick_picker_modal();
    }

    // Render with no matches for search query
    app.modals.quick_picker_search = "nonexistent_query_xyz".to_string();
    {
        let _el = app.view_quick_picker_modal();
    }

    // Render when folder has only 1 video (no other videos in folder)
    app.available_videos = vec![VideoRecord::new(1, "Anime - EP 01", &p1)];
    app.modals.quick_picker_search.clear();
    {
        let _el = app.view_quick_picker_modal();
    }

    // Render when no video is playing
    app.players[0].state.path.clear();
    {
        let _el = app.view_quick_picker_modal();
    }
}

#[test]
fn test_quick_picker_large_library_caches_and_filters_fast() {
    let (mut app, _) = new_test_app();

    let mut videos = Vec::with_capacity(5_000);
    let mut id: i64 = 1;
    // 500 siblings in target folder (above display cap) + 4,500 across 45 other folders
    for i in 0..500 {
        let path = format!("http://127.0.0.1:32400/Big%20Series/ep_{i:04}.mp4");
        let mut meta = MediaMetadata::default();
        meta.video_id = id;
        meta.title = format!("Episode {i}");
        meta.year = Some(2024);
        app.metadata_cache.insert(id, meta);
        videos.push(VideoRecord::new(id, format!("Ep {i}"), path));
        id += 1;
    }
    for folder_idx in 0..45 {
        for i in 0..100 {
            let path = format!("http://127.0.0.1:32400/Other%20Show%20{folder_idx}/ep_{i:03}.mp4");
            videos.push(VideoRecord::new(id, format!("Other {i}"), path));
            id += 1;
        }
    }

    app.available_videos = videos;
    let _ = app.update(Message::SetPlayerCount(1));
    let target_first = app.available_videos[0].path.clone();
    let target_last = app.available_videos[499].path.clone();
    app.players[0].state.path = target_first;

    let start = std::time::Instant::now();
    press_char(&mut app, "k");
    assert!(app.modals.quick_picker);
    let cache = app
        .modals
        .quick_picker_cache
        .as_ref()
        .expect("quick picker should populate entry cache on open");
    assert_eq!(cache.entries.len(), 499);
    assert_eq!(cache.entries[0].badge.as_deref(), Some("2024"));

    // Multiple renders and typing should reuse the cached snapshot without timing out.
    for _ in 0..5 {
        let _el = app.view_quick_picker_modal();
    }
    // Search for an item past the 200-row display cap; full cache must still be searchable.
    let _ = app.update(Message::QuickPickerSearchChanged("0499".to_string()));
    let _ = app.update(Message::QuickPickerSubmit);
    assert!(!app.modals.quick_picker);
    assert!(app.modals.quick_picker_cache.is_none());
    assert_eq!(app.players[0].state.path, target_last);
    assert!(
        start.elapsed() < std::time::Duration::from_secs(2),
        "opening and filtering quick picker took too long: {:?}",
        start.elapsed()
    );
}
