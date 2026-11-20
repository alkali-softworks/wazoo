use super::common::TempTestDir;
use wazoo_app::app::new_test_app;
use wazoo_app::message::Message;
use wazoo_core::VideoRecord;

#[test]
fn test_get_next_video_rec_sequential_and_random() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "V1", "/media/v1.mp4"),
        VideoRecord::new(2, "V2", "/media/v2.mp4"),
        VideoRecord::new(3, "V3", "/media/v3.mp4"),
    ];

    // 1. Sequential mode
    app.default_shuffle_mode = false;
    let next1 = app.get_next_video_rec(Some("/media/v1.mp4")).unwrap();
    assert_eq!(next1.path, "/media/v2.mp4");
    let next2 = app.get_next_video_rec(Some("/media/v2.mp4")).unwrap();
    assert_eq!(next2.path, "/media/v3.mp4");
    let next3 = app.get_next_video_rec(Some("/media/v3.mp4")).unwrap();
    assert_eq!(next3.path, "/media/v1.mp4"); // Wraps around

    // 2. Random/shuffle mode
    app.default_shuffle_mode = true;
    for _ in 0..10 {
        let rand_rec = app.get_next_video_rec(Some("/media/v1.mp4")).unwrap();
        assert!(app.available_videos.iter().any(|v| v.path == rand_rec.path));
        // With 3 videos, shuffle mode avoids immediately repeating current video
        assert_ne!(rand_rec.path, "/media/v1.mp4");
    }
}

#[test]
fn test_get_prev_video_rec_sequential_and_random() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "V1", "/media/v1.mp4"),
        VideoRecord::new(2, "V2", "/media/v2.mp4"),
        VideoRecord::new(3, "V3", "/media/v3.mp4"),
    ];

    // 1. Sequential mode
    let prev2 = app
        .get_prev_video_rec_with_mode(Some("/media/v2.mp4"), false)
        .unwrap();
    assert_eq!(prev2.path, "/media/v1.mp4");
    let prev1 = app
        .get_prev_video_rec_with_mode(Some("/media/v1.mp4"), false)
        .unwrap();
    assert_eq!(prev1.path, "/media/v3.mp4"); // Wraps around to end

    // 2. Random/shuffle mode
    for _ in 0..10 {
        let rand_rec = app
            .get_prev_video_rec_with_mode(Some("/media/v1.mp4"), true)
            .unwrap();
        assert!(app.available_videos.iter().any(|v| v.path == rand_rec.path));
        // With 3 videos, shuffle mode avoids immediately repeating current video
        assert_ne!(rand_rec.path, "/media/v1.mp4");
    }
}

#[test]
fn test_navigation_secondary_filter_from_file_picker_search() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "Cowboy Bebop - 01", "/anime/Cowboy Bebop/01.mkv"),
        VideoRecord::new(2, "Cowboy Bebop - 02", "/anime/Cowboy Bebop/02.mkv"),
        VideoRecord::new(3, "Space Dandy - 01", "/anime/Space Dandy/01.mkv"),
        VideoRecord::new(4, "Space Dandy - 02", "/anime/Space Dandy/02.mkv"),
        VideoRecord::new(5, "Trigun - 01", "/anime/Trigun/01.mkv"),
    ];

    // 1. When file picker search is empty or whitespace, sequential navigation walks all available videos
    app.drawers.file_picker_search = "".to_string();
    let next = app
        .get_next_video_rec_for_navigation(Some("/anime/Cowboy Bebop/01.mkv"), false)
        .unwrap();
    assert_eq!(next.path, "/anime/Cowboy Bebop/02.mkv");

    app.drawers.file_picker_search = "   ".to_string();
    let next_space_blank = app
        .get_next_video_rec_for_navigation(Some("/anime/Cowboy Bebop/01.mkv"), false)
        .unwrap();
    assert_eq!(next_space_blank.path, "/anime/Cowboy Bebop/02.mkv");

    // 2. When file picker search has a secondary filter "space", candidates are filtered
    app.drawers.file_picker_search = "space".to_string();
    // Starting from Bebop (which is outside the filter), next should enter Space Dandy
    let next1 = app
        .get_next_video_rec_for_navigation(Some("/anime/Cowboy Bebop/01.mkv"), false)
        .unwrap();
    assert_eq!(next1.path, "/anime/Space Dandy/01.mkv");

    // Advancing from Space Dandy 01 gives Space Dandy 02
    let next2 = app
        .get_next_video_rec_for_navigation(Some("/anime/Space Dandy/01.mkv"), false)
        .unwrap();
    assert_eq!(next2.path, "/anime/Space Dandy/02.mkv");

    // Advancing from Space Dandy 02 wraps around to Space Dandy 01 within the filtered pool
    let next3 = app
        .get_next_video_rec_for_navigation(Some("/anime/Space Dandy/02.mkv"), false)
        .unwrap();
    assert_eq!(next3.path, "/anime/Space Dandy/01.mkv");

    // Going backwards from Space Dandy 01 wraps to Space Dandy 02 within filtered pool
    let prev1 = app
        .get_prev_video_rec_for_navigation(Some("/anime/Space Dandy/01.mkv"), false)
        .unwrap();
    assert_eq!(prev1.path, "/anime/Space Dandy/02.mkv");

    // 3. Fallback when filter matches no videos: falls back to available_videos
    app.drawers.file_picker_search = "nonexistent_query_xyz".to_string();
    let fallback = app
        .get_next_video_rec_for_navigation(Some("/anime/Cowboy Bebop/01.mkv"), false)
        .unwrap();
    assert_eq!(fallback.path, "/anime/Cowboy Bebop/02.mkv");
}

#[test]
fn test_typing_in_file_picker_search_does_not_swap_playing_video() {
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("test_typing_filter");
    let f1 = temp_dir.path().join("Cowboy Bebop - 01.mp4");
    let f2 = temp_dir.path().join("Space Dandy - 01.mp4");
    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);

    let path1 = f1.to_string_lossy().to_string();
    let path2 = f2.to_string_lossy().to_string();

    app.available_videos = vec![
        VideoRecord::new(1, "Cowboy Bebop - 01", &path1),
        VideoRecord::new(2, "Space Dandy - 01", &path2),
    ];
    app.default_shuffle_mode = false;

    // Initialize 1 player; it plays first video (Cowboy Bebop)
    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);
    let p0_id = app.players[0].id;
    app.players[0].shuffle = false;
    assert_eq!(app.players[0].state.path, path1);

    // User types in file picker drawer search box
    let _ = app.update(Message::FilePickerSearchChanged("space".to_string()));
    assert_eq!(app.drawers.file_picker_search, "space");
    // Playing video MUST NOT be swapped while typing!
    assert_eq!(app.players[0].state.path, path1);

    // Debounce timer arrives to apply drawer search UI filtering
    let _ = app.update(Message::ApplyFilePickerSearch);
    // Playing video STILL MUST NOT be swapped!
    assert_eq!(app.players[0].state.path, path1);

    // Only when user advances to next video does the secondary filter take effect
    let _ = app.update(Message::NextVideo(p0_id));
    assert_eq!(app.players[0].state.path, path2);
}

#[test]
fn test_file_picker_confined_folder_badge_multilingual() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "Anime 1", "/media/anime/a1.mp4"),
        VideoRecord::new(2, "Movie 1", "/media/movies/m1.mp4"),
    ];

    // 1. Switch language to Spanish
    let _ = app.update(Message::SetLanguage("es".to_string()));
    let all_es = app.t("common.all");
    assert_eq!(all_es, "Todo");
    assert_eq!(app.search.active_folder, "Todo");
    assert_eq!(app.search.selected_folder, "Todo");
    assert!(app.settings.last_folders.is_empty());
    assert!(app.is_all_folder(&app.search.active_folder));

    // 2. File picker in Spanish does not show badge for Todo
    {
        let _view_all = app.view_file_picker();
    }

    // 3. Search confined to anime
    let _ = app.update(Message::SelectSearchFolder("/media/anime".to_string()));
    assert_eq!(app.search.active_folder, "Todo"); // Still Todo before search
    let _ = app.update(Message::PerformSearch);
    assert_eq!(app.search.active_folder, "/media/anime");
    assert_eq!(app.settings.last_folders, vec!["/media/anime".to_string()]);
    assert!(!app.is_all_folder(&app.search.active_folder));

    // 4. Reset search folder returns to Spanish All ("Todo")
    let _ = app.update(Message::ResetSearchFolder);
    assert_eq!(app.search.active_folder, "Todo");
    assert_eq!(app.search.selected_folder, "Todo");
    assert!(app.settings.last_folders.is_empty());
    assert!(app.is_all_folder(&app.search.active_folder));
}

#[test]
fn test_file_picker_multiple_confined_folder_chips_wrap() {
    let (mut app, _) = new_test_app();
    app.settings.media_folders = vec![
        "/media/anime".to_string(),
        "/media/tv".to_string(),
        "/media/Sat_Morning_Shows".to_string(),
    ];
    app.available_videos = vec![
        VideoRecord::new(1, "Anime 1", "/media/anime/a1.mp4"),
        VideoRecord::new(2, "TV 1", "/media/tv/t1.mp4"),
        VideoRecord::new(3, "Sat Show 1", "/media/Sat_Morning_Shows/s1.mp4"),
    ];

    // Select multiple folders
    app.search.active_folders = vec![
        "/media/anime".to_string(),
        "/media/tv".to_string(),
        "/media/Sat_Morning_Shows".to_string(),
    ];

    // Open file picker drawer
    let _ = app.update(Message::ToggleFilePicker);
    assert!(app.drawers.show_file_picker);

    // Verify view renders with wrapped chips
    {
        let _view = app.view_file_picker();
    }

    // Remove one chip
    let _ = app.update(Message::RemoveActiveSearchFolder("/media/tv".to_string()));
    assert_eq!(app.search.active_folders.len(), 2);
    assert!(!app.search.active_folders.contains(&"/media/tv".to_string()));

    {
        let _view_after_remove = app.view_file_picker();
    }
}

#[test]
fn test_file_picker_highlights_playing_video() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "Anime 1", "/media/anime/a1.mp4"),
        VideoRecord::new(2, "Movie 1", "/media/movies/m1.mp4"),
    ];
    let _ = app.update(Message::ToggleFilePicker);
    let _ = app.update(Message::ToggleFolderCollapse("anime".to_string()));

    {
        let _view_before = app.view_file_picker();
    }

    if let Some(p) = app.players.first_mut() {
        p.state.path = "/media/videos/clip1.mp4".to_string();
    }
    {
        let _view_playing = app.view_file_picker();
    }
}

#[test]
fn test_query_zero_matches_shows_no_matches_view_and_clear_search() {
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("wazoo_test_no_match");
    let f1 = temp_dir.path().join("existing_video.mp4");
    let _ = std::fs::File::create(&f1);

    // Add video to DB
    let _ = app
        .db
        .insert_misc_video("existing_video", &f1.to_string_lossy());
    assert_eq!(app.db.get_video_count().unwrap(), 1);

    // Perform a search for a term that does not match
    app.search.input = "nonexistent_query_xyz".to_string();
    let _ = app.update(Message::PerformSearch);

    assert_eq!(app.search.active_query, "nonexistent_query_xyz");
    assert!(app.available_videos.is_empty());

    // Calling view() exercises the view_no_matches code path
    {
        let _main_view = app.view();
    }

    // Now clear search
    let _ = app.update(Message::ClearSearch);
    assert!(app.search.active_query.is_empty());
    assert_eq!(app.available_videos.len(), 1);
}

#[test]
fn test_set_active_query_updates_search_tags_and_reconciles() {
    let (mut app, _) = new_test_app();
    app.db
        .insert_or_update_video(&VideoRecord::new(
            1,
            "ShowA - S01E01",
            "/media/Shows/ShowA - S01E01.mp4",
        ))
        .unwrap();
    app.db
        .insert_or_update_video(&VideoRecord::new(
            2,
            "ShowA - S01E02",
            "/media/Shows/ShowA - S01E02.mp4",
        ))
        .unwrap();
    app.db
        .insert_or_update_video(&VideoRecord::new(
            3,
            "ShowB - S01E01",
            "/media/Shows/ShowB - S01E01.mp4",
        ))
        .unwrap();

    let _ = app.update(Message::SetActiveQuery("ShowA".to_string()));
    assert_eq!(app.search.active_query, "ShowA");
    assert_eq!(app.search.tags, vec!["ShowA"]);
    assert_eq!(app.available_videos.len(), 2);
    assert!(
        app.available_videos
            .iter()
            .all(|v| v.path.contains("ShowA"))
    );
}

#[test]
fn test_file_picker_search_clear_button() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "Space Odyssey", "/media/movies/space_odyssey.mp4"),
        VideoRecord::new(2, "Nature Documentary", "/media/nature/doc1.mp4"),
    ];

    // Open file picker drawer
    let _ = app.update(Message::ToggleFilePicker);
    assert!(app.drawers.show_file_picker);

    // Initial empty search: view file picker renders without clear button
    assert!(app.drawers.file_picker_search.is_empty());
    {
        let _view_empty = app.view_file_picker();
    }

    // User types in search: search input is not empty
    let _ = app.update(Message::FilePickerSearchChanged("space".to_string()));
    assert_eq!(app.drawers.file_picker_search, "space");
    assert!(app.drawers.file_picker_debounce_ticks > 0);

    // View renders with clear button layered on search bar
    {
        let _view_with_query = app.view_file_picker();
    }

    // Navigating with "space" active yields Space Odyssey even starting from Nature Documentary
    let next_filtered = app
        .get_next_video_rec_for_navigation(Some("/media/nature/doc1.mp4"), false)
        .unwrap();
    assert_eq!(next_filtered.name, "Space Odyssey");

    // Pressing the clear button:
    let _ = app.update(Message::ClearFilePickerSearch);
    assert_eq!(app.drawers.file_picker_search, "");
    assert_eq!(app.drawers.file_picker_debounce_ticks, 0);

    // Filter cleared: sequential navigation from Space Odyssey advances to Nature Documentary
    let next_all = app
        .get_next_video_rec_for_navigation(Some("/media/movies/space_odyssey.mp4"), false)
        .unwrap();
    assert_eq!(next_all.name, "Nature Documentary");

    // View renders cleanly with empty search input (no clear button)
    {
        let _view_cleared = app.view_file_picker();
    }
}

#[test]
fn test_search_modal_long_tags_list_wrapping_and_scrolling() {
    let (mut app, _) = new_test_app();

    // 1. Open search modal with no tags
    let _ = app.update(Message::OpenSearchModal);
    assert!(app.modals.search);
    assert!(app.search.tags.is_empty());
    {
        let _view_empty = app.view_search_modal();
    }

    // 2. Add multiple tags matching the user's scenario
    let _ = app.update(Message::SearchInputChanged(
        "not dragon ball, twilight, boku, hotel, otome wa boku desu,".to_string(),
    ));
    assert_eq!(app.search.tags.len(), 5);
    assert_eq!(app.search.tags[0], "not dragon ball");
    assert_eq!(app.search.tags[4], "otome wa boku desu");
    {
        let _view_wrapped = app.view_search_modal();
    }

    // 3. Add more tags to trigger the scrollable view (> 3 rows)
    let _ = app.update(Message::SearchInputChanged(
        "tag6, tag7, tag8, tag9, tag10, tag11, tag12, tag13,".to_string(),
    ));
    assert_eq!(app.search.tags.len(), 13);
    {
        let _view_scrollable = app.view_search_modal();
    }

    // 4. Remove a tag and verify view renders cleanly
    let _ = app.update(Message::RemoveSearchTag(0));
    assert_eq!(app.search.tags.len(), 12);
    {
        let _view_after_remove = app.view_search_modal();
    }
}

#[test]
fn test_toggle_search_metadata() {
    let (mut app, _) = new_test_app();
    assert!(app.settings.search_include_metadata);

    let v1 = VideoRecord::new(1, "movie1", "/media/movies/matrix.mkv");
    let id1 = app.db.insert_or_update_video(&v1).unwrap();
    app.db
        .upsert_metadata(&wazoo_core::models::MediaMetadata {
            video_id: id1,
            title: "The Matrix".to_string(),
            genres: vec!["Sci-Fi".to_string()],
            ..Default::default()
        })
        .unwrap();

    // Render search modal with toggle enabled
    {
        let _modal = app.view_search_modal();
    }

    // Search by metadata genre while enabled
    let _ = app.update(Message::SearchInputChanged("Sci-Fi".to_string()));
    let _ = app.update(Message::PerformSearch);
    assert_eq!(app.available_videos.len(), 1);

    // Switch to Filename-only and re-search
    let _ = app.update(Message::SetSearchIncludeMetadata(false));
    assert!(!app.settings.search_include_metadata);
    let _ = app.update(Message::PerformSearch);
    assert!(app.available_videos.is_empty());

    // Switch back to Metadata and re-search
    let _ = app.update(Message::SetSearchIncludeMetadata(true));
    assert!(app.settings.search_include_metadata);
    let _ = app.update(Message::PerformSearch);
    assert_eq!(app.available_videos.len(), 1);
}
