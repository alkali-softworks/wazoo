use super::common::{TempTestDir, press_char};
use wazoo_app::app::{SettingsTab, new_test_app};
use wazoo_app::message::Message;
use wazoo_core::{MediaMetadata, PlexExplorerTab, PlexRawItem, VideoRecord};

#[test]
fn test_plex_explorer_modal_e_key_escape_and_actions() {
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("test_plex_explorer");

    let f1 = temp_dir.path().join("Frailty (2001).mp4");
    let f2 = temp_dir.path().join("The Matrix (1999).mkv");
    let f3 = temp_dir.path().join("The Shining (1980).mp4");
    let f4 = temp_dir.path().join("Blade Runner (1982).mp4");
    let f5 = temp_dir.path().join("Dark Star (1974).mp4");
    let f6 = temp_dir.path().join("Nosferatu (1922).mp4");
    for f in [&f1, &f2, &f3, &f4, &f5, &f6] {
        let _ = std::fs::File::create(f);
    }

    let p1 = f1.to_string_lossy().to_string();
    let p2 = f2.to_string_lossy().to_string();
    let p3 = f3.to_string_lossy().to_string();
    let p4 = f4.to_string_lossy().to_string();
    let p5 = f5.to_string_lossy().to_string();
    let p6 = f6.to_string_lossy().to_string();

    let v1 = VideoRecord::new(1, "Frailty (2001)", &p1);
    let v2 = VideoRecord::new(2, "The Matrix (1999)", &p2);
    let v3 = VideoRecord::new(3, "The Shining (1980)", &p3);
    let v4 = VideoRecord::new(4, "Blade Runner (1982)", &p4);
    let v5 = VideoRecord::new(5, "Dark Star (1974)", &p5);
    let v6 = VideoRecord::new(6, "Nosferatu (1922)", &p6);

    let mut videos = vec![
        v1.clone(),
        v2.clone(),
        v3.clone(),
        v4.clone(),
        v5.clone(),
        v6.clone(),
    ];
    for v in &mut videos {
        v.id = app.db.insert_or_update_video(v).unwrap();
    }
    app.available_videos = videos.clone();

    let metas = vec![
        MediaMetadata {
            video_id: videos[0].id,
            title: "Frailty".to_string(),
            year: Some(2001),
            rating: Some(7.6),
            genres: vec![
                "Drama".to_string(),
                "Thriller".to_string(),
                "Crime".to_string(),
            ],
            directors: vec!["Bill Paxton".to_string()],
            actors: vec!["Matthew McConaughey".to_string()],
            tagline: Some("No soul is safe.".to_string()),
            ..Default::default()
        },
        MediaMetadata {
            video_id: videos[1].id,
            title: "The Matrix".to_string(),
            year: Some(1999),
            rating: Some(8.7),
            genres: vec!["Action".to_string(), "Sci-Fi".to_string()],
            directors: vec!["Wachowski".to_string()],
            actors: vec!["Keanu Reeves".to_string()],
            tagline: Some("Welcome to the Real World.".to_string()),
            ..Default::default()
        },
        MediaMetadata {
            video_id: videos[2].id,
            title: "The Shining".to_string(),
            year: Some(1980),
            rating: Some(8.4),
            genres: vec!["Horror".to_string(), "Drama".to_string()],
            directors: vec!["Stanley Kubrick".to_string()],
            actors: vec!["Jack Nicholson".to_string()],
            tagline: Some("A masterpiece of modern horror.".to_string()),
            ..Default::default()
        },
        MediaMetadata {
            video_id: videos[3].id,
            title: "Blade Runner".to_string(),
            year: Some(1982),
            rating: Some(8.1),
            genres: vec!["Science Fiction".to_string(), "Thriller".to_string()],
            directors: vec!["Ridley Scott".to_string()],
            actors: vec!["Harrison Ford".to_string()],
            tagline: Some("Man has made his match... now it's his problem.".to_string()),
            ..Default::default()
        },
        MediaMetadata {
            video_id: videos[4].id,
            title: "Dark Star".to_string(),
            year: Some(1974),
            rating: Some(7.4),
            genres: vec!["Sci-Fi".to_string(), "Comedy".to_string()],
            directors: vec!["John Carpenter".to_string()],
            ..Default::default()
        },
        MediaMetadata {
            video_id: videos[5].id,
            title: "Nosferatu".to_string(),
            year: Some(1922),
            rating: Some(7.9),
            genres: vec!["Horror".to_string(), "Fantasy".to_string()],
            directors: vec!["F.W. Murnau".to_string()],
            ..Default::default()
        },
    ];
    for m in &metas {
        app.db.upsert_metadata(m).unwrap();
        app.metadata_cache.insert(m.video_id, m.clone());
    }

    let _ = app.update(Message::SetPlayerCount(1));
    assert!(!app.modals.plex_explorer);
    assert!(!app.modals.is_any_open());

    // 1. Press 'e' key to open Plex Explorer modal (defaults to 'All')
    press_char(&mut app, "e");
    assert!(app.modals.plex_explorer);
    assert!(app.modals.is_any_open());
    assert_eq!(app.modals.plex_explorer_tab, PlexExplorerTab::All);
    assert_eq!(app.modals.plex_explorer_search, "");
    let all_items = app.plex_explorer_items();
    assert_eq!(all_items.len(), 6);

    // 2. Tab switching: Top Rated
    let _ = app.update(Message::PlexExplorerSelectTab(PlexExplorerTab::TopRated));
    assert_eq!(app.modals.plex_explorer_tab, PlexExplorerTab::TopRated);
    let items = app.plex_explorer_items();
    assert_eq!(items.len(), 6);
    // Highest rated first: The Matrix (8.7), The Shining (8.4), Blade Runner (8.1), Nosferatu (7.9), Frailty (7.6), Dark Star (7.4)
    assert_eq!(items[0].title, "The Matrix");
    assert_eq!(items[1].title, "The Shining");
    assert_eq!(items[2].title, "Blade Runner");
    assert_eq!(items[3].title, "Nosferatu");
    assert_eq!(items[4].title, "Frailty");
    assert_eq!(items[5].title, "Dark Star");

    // 2b. Toggling genre or decade while in Top Rated must NOT jump to 'All'
    let _ = app.update(Message::PlexExplorerToggleGenre(Some("Sci-Fi".to_string())));
    assert_eq!(
        app.modals.plex_explorer_tab,
        PlexExplorerTab::TopRated,
        "Toggling genre should not change active tab from TopRated"
    );
    // Top Rated list remains unfiltered by genre
    let tr_items = app.plex_explorer_items();
    assert_eq!(tr_items.len(), 6);

    // 2c. Switching back to 'All' tab applies the genre filter
    let _ = app.update(Message::PlexExplorerSelectTab(PlexExplorerTab::All));
    assert_eq!(app.modals.plex_explorer_tab, PlexExplorerTab::All);
    // Genre "Sci-Fi" matches "Sci-Fi" (The Matrix, Dark Star) AND "Science Fiction" (Blade Runner)
    let scifi_items = app.plex_explorer_items();
    assert_eq!(scifi_items.len(), 3);
    let scifi_titles: Vec<&str> = scifi_items.iter().map(|i| i.title.as_str()).collect();
    assert!(scifi_titles.contains(&"The Matrix"));
    assert!(scifi_titles.contains(&"Blade Runner"));
    assert!(scifi_titles.contains(&"Dark Star"));

    // Clear genre filter
    let _ = app.update(Message::PlexExplorerToggleGenre(None));
    assert!(app.modals.plex_explorer_genre.is_none());
    assert_eq!(app.plex_explorer_items().len(), 6);

    // 3. Sub-filter by Decade: 1990s
    let _ = app.update(Message::PlexExplorerSelectTab(PlexExplorerTab::Decade(1990)));
    let items = app.plex_explorer_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "The Matrix");

    // 3b. Sub-filter by Decade: 1970s (matches Dark Star 1974)
    let _ = app.update(Message::PlexExplorerSelectTab(PlexExplorerTab::Decade(1970)));
    let items = app.plex_explorer_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Dark Star");

    // 3c. Sub-filter by pre-30s: 1920s (matches Nosferatu 1922)
    let _ = app.update(Message::PlexExplorerSelectTab(PlexExplorerTab::Decade(1920)));
    let items = app.plex_explorer_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Nosferatu");

    // 4. Sub-filter by Genre: Thriller (Frailty + Blade Runner)
    let _ = app.update(Message::PlexExplorerSelectTab(PlexExplorerTab::Genre(
        "Thriller".to_string(),
    )));
    let items = app.plex_explorer_items();
    assert_eq!(items.len(), 2);
    let thriller_titles: Vec<&str> = items.iter().map(|i| i.title.as_str()).collect();
    assert!(thriller_titles.contains(&"Frailty"));
    assert!(thriller_titles.contains(&"Blade Runner"));

    // 5. Switch to All and test simultaneous multi-filtering: 80s Horror
    let _ = app.update(Message::PlexExplorerSelectTab(PlexExplorerTab::All));
    let _ = app.update(Message::PlexExplorerToggleGenre(Some("Horror".to_string())));
    let _ = app.update(Message::PlexExplorerToggleDecade(Some(1980)));
    assert_eq!(app.modals.plex_explorer_genre.as_deref(), Some("Horror"));
    assert_eq!(app.modals.plex_explorer_decade, Some(1980));
    let items = app.plex_explorer_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "The Shining");

    // 5b. Simultaneous multi-filtering: 70s Sci-Fi (Dark Star)
    let _ = app.update(Message::PlexExplorerToggleGenre(Some("Sci-Fi".to_string())));
    let _ = app.update(Message::PlexExplorerToggleDecade(Some(1970)));
    let items = app.plex_explorer_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Dark Star");

    // 6. Search filtering within active multi-filter
    let _ = app.update(Message::PlexExplorerSearchChanged("Carpenter".to_string()));
    assert_eq!(app.modals.plex_explorer_search, "Carpenter");
    let items = app.plex_explorer_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Dark Star");

    // 6b. Search filtering with "scifi" alias finds "Science Fiction" and "Sci-Fi"
    let _ = app.update(Message::PlexExplorerToggleGenre(None));
    let _ = app.update(Message::PlexExplorerToggleDecade(None));
    let _ = app.update(Message::PlexExplorerSearchChanged("scifi".to_string()));
    let search_items = app.plex_explorer_items();
    assert_eq!(search_items.len(), 3);
    let search_titles: Vec<&str> = search_items.iter().map(|i| i.title.as_str()).collect();
    assert!(search_titles.contains(&"The Matrix"));
    assert!(search_titles.contains(&"Blade Runner"));
    assert!(search_titles.contains(&"Dark Star"));

    // 6c. Search filtering with decade query like "70s"
    let _ = app.update(Message::PlexExplorerSearchChanged("70s".to_string()));
    let search_items = app.plex_explorer_items();
    assert_eq!(search_items.len(), 1);
    assert_eq!(search_items[0].title, "Dark Star");

    // Reset search
    let _ = app.update(Message::ClearPlexExplorerSearch);

    // 7. Submitting via Enter plays the first matching file and closes modal
    let _ = app.update(Message::PlexExplorerSubmit);
    assert!(!app.modals.plex_explorer);
    assert_eq!(app.players[0].state.path, p2);

    // 8. Verify Surprise Me produces fresh randomized list on every tab selection
    app.modals.plex_explorer = true;
    let _ = app.update(Message::PlexExplorerSelectTab(PlexExplorerTab::Random));
    assert_eq!(app.modals.plex_explorer_tab, PlexExplorerTab::Random);
    let seed1 = app.modals.plex_explorer_random_seed;
    assert_ne!(seed1, 0);
    let items_rand1 = app.plex_explorer_items();
    assert_eq!(items_rand1.len(), 6);
    // Stable within same seed
    let items_rand1_repeat = app.plex_explorer_items();
    let titles1: Vec<&str> = items_rand1.iter().map(|i| i.title.as_str()).collect();
    let titles1_repeat: Vec<&str> = items_rand1_repeat.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(titles1, titles1_repeat);

    // Clicking Surprise Me again updates seed to generate a fresh randomized list
    let _ = app.update(Message::PlexExplorerSelectTab(PlexExplorerTab::Random));
    let seed2 = app.modals.plex_explorer_random_seed;
    assert_ne!(seed1, seed2);

    // 9. Verify view rendering across All, TopRated, Recently Added, On Deck, and SurpriseMe
    app.modals.plex_explorer_genre = Some("Horror".to_string());
    app.modals.plex_explorer_decade = Some(1980);
    for tab in [
        PlexExplorerTab::All,
        PlexExplorerTab::TopRated,
        PlexExplorerTab::RecentlyAdded,
        PlexExplorerTab::OnDeck,
        PlexExplorerTab::Random,
    ] {
        app.modals.plex_explorer_tab = tab;
        let _el = app.view_plex_explorer_modal();
    }

    // 9b. Narrowing available_videos (main global query) must NOT limit Plex Explorer
    app.available_videos = vec![v1.clone()];
    app.modals.plex_explorer_genre = None;
    app.modals.plex_explorer_decade = None;
    app.modals.plex_explorer_search.clear();

    for tab in [
        PlexExplorerTab::All,
        PlexExplorerTab::TopRated,
        PlexExplorerTab::RecentlyAdded,
        PlexExplorerTab::Random,
    ] {
        let _ = app.update(Message::PlexExplorerSelectTab(tab));
        assert_eq!(app.plex_explorer_items().len(), 6);
    }

    let live_raw = vec![PlexRawItem {
        rating_key: "rk2".to_string(),
        title: "The Matrix".to_string(),
        series_title: None,
        season_number: None,
        episode_number: None,
        year: Some(1999),
        tagline: None,
        summary: None,
        content_rating: None,
        duration_ms: None,
        view_offset_ms: Some(120_000),
        rating: Some(8.7),
        studio: None,
        genres: vec![],
        directors: vec![],
        actors: vec![],
        thumb_url: None,
        art_url: None,
        file_paths: vec![p2.clone()],
    }];
    let _ = app.update(Message::PlexExplorerSelectTab(PlexExplorerTab::OnDeck));
    let _ = app.update(Message::PlexExplorerLiveLoaded(
        PlexExplorerTab::OnDeck,
        live_raw,
    ));
    let on_deck_items = app.plex_explorer_items();
    assert_eq!(on_deck_items.len(), 1);
    assert_eq!(on_deck_items[0].title, "The Matrix");

    // 10. Press Escape closes the modal and clears filters
    let _ = app.update(Message::EscapePressed);
    assert!(!app.modals.plex_explorer);
    assert!(app.modals.plex_explorer_genre.is_none());
    assert!(app.modals.plex_explorer_decade.is_none());
}

#[test]
fn test_plex_explorer_modal_unauthenticated_state() {
    let (mut app, _) = new_test_app();
    assert!(!app.settings.plex.enabled);
    assert!(app.settings.plex.token.is_empty());

    // 1. Open Plex Explorer when unauthenticated (default settings)
    let _ = app.update(Message::OpenPlexExplorerModal);
    assert!(app.modals.plex_explorer);
    assert_eq!(app.plex_explorer_items().len(), 0);

    // 2. Rendering modal in unauthenticated state succeeds
    {
        let _view = app.view_plex_explorer_modal();
    }

    // 3. Trigger OpenPlexSettingsModal (e.g. from the notice button or header warning pill)
    let _ = app.update(Message::OpenPlexSettingsModal);
    assert!(!app.modals.plex_explorer, "Plex Explorer modal should close");
    assert!(app.modals.settings, "Settings modal should open");
    assert_eq!(
        app.modals.settings_tab,
        SettingsTab::Plex,
        "Settings should navigate to the Plex tab"
    );

    // 4. Test when token is set, but Plex is disabled
    app.settings.plex.token = "my_sample_token".to_string();
    app.settings.plex.enabled = false;
    let _ = app.update(Message::OpenPlexExplorerModal);
    assert!(app.modals.plex_explorer);
    {
        let _view_disabled = app.view_plex_explorer_modal();
    }

    // 5. Test when enabled with token, but no synced items in metadata cache
    app.settings.plex.enabled = true;
    {
        let _view_no_items = app.view_plex_explorer_modal();
    }

    // 6. Test with a query that returns no matches
    app.modals.plex_explorer_search = "xyz123nonexistent".to_string();
    {
        let _view_query = app.view_plex_explorer_modal();
    }
}
