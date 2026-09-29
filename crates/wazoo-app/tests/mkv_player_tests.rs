use std::path::PathBuf;
use wazoo_app::app::WazooApp;
use wazoo_app::cli::CliArgs;
use wazoo_app::message::Message;
use wazoo_core::{ConfigManager, Database, VideoRecord};

fn setup_isolated_env() -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir().join(format!(
        "wazoo_mkv_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&base);
    let base = std::fs::canonicalize(&base).unwrap_or(base);
    let config_dir = base.join("config");
    let media_dir = base.join("media");
    let _ = std::fs::create_dir_all(&config_dir);
    let _ = std::fs::create_dir_all(&media_dir);
    (config_dir, media_dir)
}

#[test]
fn test_mkv_default_player_adds_to_misc_folder_and_plays() {
    let (config_dir, media_dir) = setup_isolated_env();

    // Setup library folder and pre-existing library video
    let library_dir = media_dir.join("library");
    std::fs::create_dir_all(&library_dir).unwrap();
    let lib_vid = library_dir.join("library_show.mp4");
    std::fs::File::create(&lib_vid).unwrap();

    let config_mgr = ConfigManager::with_dirs(config_dir.clone(), config_dir);
    let db = Database::open_in_memory().unwrap();

    // Populate prior query in settings
    let mut settings = config_mgr.load_settings();
    settings.media_folders = vec![library_dir.to_string_lossy().to_string()];
    settings.last_query = "library".to_string();
    settings.last_folders = vec![library_dir.to_string_lossy().to_string()];
    config_mgr.save_settings(&settings).unwrap();

    // Insert library video into DB
    db.insert_or_update_video(&VideoRecord::new(
        1,
        "Library Show",
        lib_vid.to_string_lossy(),
    ))
    .unwrap();

    // External MKV file (e.g. from Downloads)
    let downloads_dir = media_dir.join("downloads");
    std::fs::create_dir_all(&downloads_dir).unwrap();
    let ext_mkv = downloads_dir.join("my_downloaded_movie.mkv");
    std::fs::File::create(&ext_mkv).unwrap();

    // Launch app with the external MKV file
    let cli = CliArgs::from_file(ext_mkv.clone());
    let (mut app, _) = WazooApp::new_with_backend(cli, config_mgr, db);

    // 1. Verify file was added to DB as 'Misc'
    assert_eq!(app.db.get_misc_video_count().unwrap(), 1);
    assert!(app.db.has_misc_videos().unwrap());

    let misc_videos = app.db.search_videos("", &["Misc".to_string()]).unwrap();
    assert_eq!(misc_videos.len(), 1);
    assert_eq!(misc_videos[0].folder.as_deref(), Some("Misc"));

    // 2. Verify prior query was NOT overwritten
    assert_eq!(app.settings.last_query, "library");
    assert_eq!(
        app.settings.last_folders,
        vec![library_dir.to_string_lossy().to_string()]
    );

    // 3. Verify available_videos contains the prior query's videos
    assert_eq!(app.available_videos.len(), 1);
    assert_eq!(app.available_videos[0].name, "Library Show");

    // 4. Verify Player 1 was spawned playing the external MKV file directly
    assert_eq!(app.players.len(), 1);
    let player = &app.players[0];
    let canon_ext = std::fs::canonicalize(&ext_mkv).unwrap();
    let canon_player = std::fs::canonicalize(&player.state.path)
        .unwrap_or_else(|_| PathBuf::from(&player.state.path));
    assert_eq!(canon_player, canon_ext);
    assert!(!player.state.is_muted);
    assert_eq!(player.state.volume, 1.0);

    // 5. Advance Player 1 to next video -> should continue with prior query!
    let _ = app.update(Message::NextVideo(player.id));
    assert_eq!(app.players.len(), 1);
    let updated_player = &app.players[0];
    let canon_lib = std::fs::canonicalize(&lib_vid).unwrap();
    let canon_updated = std::fs::canonicalize(&updated_player.state.path)
        .unwrap_or_else(|_| PathBuf::from(&updated_player.state.path));
    assert_eq!(canon_updated, canon_lib);
}

#[test]
fn test_mkv_already_in_media_folder_not_added_to_misc() {
    let (config_dir, media_dir) = setup_isolated_env();

    let anime_dir = media_dir.join("anime");
    std::fs::create_dir_all(&anime_dir).unwrap();
    let anime_file = anime_dir.join("naruto_ep01.mkv");
    std::fs::File::create(&anime_file).unwrap();

    let config_mgr = ConfigManager::with_dirs(config_dir.clone(), config_dir);
    let db = Database::open_in_memory().unwrap();

    let mut settings = config_mgr.load_settings();
    settings.media_folders = vec![anime_dir.to_string_lossy().to_string()];
    config_mgr.save_settings(&settings).unwrap();

    db.insert_or_update_video(&VideoRecord::new(
        1,
        "Naruto 01",
        anime_file.to_string_lossy(),
    ))
    .unwrap();

    // Launch app opening a file that is already inside configured media_folders
    let cli = CliArgs::from_file(anime_file.clone());
    let (app, _) = WazooApp::new_with_backend(cli, config_mgr, db);

    // Must NOT be added to Misc
    assert_eq!(app.db.get_misc_video_count().unwrap(), 0);
    assert!(!app.db.has_misc_videos().unwrap());

    // Still played directly
    assert_eq!(app.players.len(), 1);
    let canon = std::fs::canonicalize(&anime_file).unwrap();
    let canon_player = std::fs::canonicalize(&app.players[0].state.path)
        .unwrap_or_else(|_| PathBuf::from(&app.players[0].state.path));
    assert_eq!(canon_player, canon);
}

#[test]
fn test_clear_misc_videos_wipe_button() {
    let (config_dir, media_dir) = setup_isolated_env();

    let f1 = media_dir.join("c1.mkv");
    let f2 = media_dir.join("c2.mkv");
    std::fs::File::create(&f1).unwrap();
    std::fs::File::create(&f2).unwrap();

    let config_mgr = ConfigManager::with_dirs(config_dir.clone(), config_dir);
    let db = Database::open_in_memory().unwrap();

    db.insert_misc_video("Clip 1", &f1.to_string_lossy())
        .unwrap();
    db.insert_misc_video("Clip 2", &f2.to_string_lossy())
        .unwrap();
    db.insert_or_update_video(&VideoRecord::new(0, "Regular", "/media/regular.mp4"))
        .unwrap();

    assert_eq!(db.get_misc_video_count().unwrap(), 2);
    assert_eq!(db.get_video_count().unwrap(), 3);

    let (mut app, _) = WazooApp::new_with_backend(None, config_mgr, db);

    // Dispatch ClearMiscVideos message
    let _ = app.update(Message::ClearMiscVideos);

    // Verify all Misc videos were wiped
    assert_eq!(app.db.get_misc_video_count().unwrap(), 0);
    assert!(!app.db.has_misc_videos().unwrap());
    assert_eq!(app.db.get_video_count().unwrap(), 1);
    assert!(
        app.overlay
            .toast_message
            .unwrap()
            .contains("Cleared 2 miscellaneous files")
    );
}
