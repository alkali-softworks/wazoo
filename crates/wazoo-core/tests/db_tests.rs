use wazoo_core::{Database, VideoRecord};

#[test]
fn test_db_operations() {
    let db = Database::open_in_memory().unwrap();
    assert_eq!(db.get_video_count().unwrap(), 0);

    let video = VideoRecord::new(0, "Ambient Video 1", "/media/ambient1.mp4");

    db.insert_or_update_video(&video).unwrap();
    assert_eq!(db.get_video_count().unwrap(), 1);

    let search_results = db.search_videos("ambient", &[]).unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].name, "Ambient Video 1");

    let mut db = db;
    let pruned = db.prune_missing_videos(&[]).unwrap();
    assert_eq!(pruned, 1);
    assert_eq!(db.get_video_count().unwrap(), 0);
}

#[test]
fn test_remove_videos_in_folder() {
    let mut db = Database::open_in_memory().unwrap();
    db.batch_insert_videos(&[
        VideoRecord::new(0, "Vid 1", "/home/user/media/folder_a/1.mp4"),
        VideoRecord::new(0, "Vid 2", "/home/user/media/folder_a/sub/2.mp4"),
        VideoRecord::new(0, "Vid 3", "/home/user/media/folder_b/3.mp4"),
        VideoRecord::new(0, "Vid 4", "/home/user/media/folder_a_other/4.mp4"),
    ])
    .unwrap();
    assert_eq!(db.get_video_count().unwrap(), 4);

    // Remove folder_a
    let removed = db
        .remove_videos_in_folder("/home/user/media/folder_a")
        .unwrap();
    assert_eq!(removed, 2);
    assert_eq!(db.get_video_count().unwrap(), 2);

    let remaining = db.get_all_videos().unwrap();
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining[0].name, "Vid 3");
    assert_eq!(remaining[1].name, "Vid 4");
}

#[test]
fn test_search_videos_filtering_for_reconciliation() {
    let mut db = Database::open_in_memory().unwrap();
    db.batch_insert_videos(&[
        VideoRecord::new(0, "Breaking Bad S01E01", "/media/BreakingBad/S01E01.mp4"),
        VideoRecord::new(0, "Breaking Bad S01E02", "/media/BreakingBad/S01E02.mp4"),
        VideoRecord::new(
            0,
            "Game of Thrones S01E01",
            "/media/GameOfThrones/S01E01.mp4",
        ),
    ])
    .unwrap();

    // Search for Breaking Bad
    let bb_results = db.search_videos("Breaking Bad", &[]).unwrap();
    assert_eq!(bb_results.len(), 2);
    assert!(
        bb_results
            .iter()
            .any(|v| v.path == "/media/BreakingBad/S01E01.mp4")
    );
    assert!(
        bb_results
            .iter()
            .any(|v| v.path == "/media/BreakingBad/S01E02.mp4")
    );

    // Currently playing GoT does not exist in the new queried list of files
    let got_playing_path = "/media/GameOfThrones/S01E01.mp4";
    assert!(!bb_results.iter().any(|v| v.path == got_playing_path));
}

#[test]
fn test_search_videos_multiple_folders_mix_and_match() {
    let mut db = Database::open_in_memory().unwrap();
    db.batch_insert_videos(&[
        VideoRecord::new(0, "Breaking Bad S01E01", "/media/BreakingBad/S01E01.mp4"),
        VideoRecord::new(
            0,
            "Game of Thrones S01E01",
            "/media/GameOfThrones/S01E01.mp4",
        ),
        VideoRecord::new(0, "The Wire S01E01", "/media/TheWire/S01E01.mp4"),
    ])
    .unwrap();

    // 1. Search with multiple folders: BreakingBad and TheWire (omit GameOfThrones)
    let folders = vec![
        "/media/BreakingBad".to_string(),
        "/media/TheWire".to_string(),
    ];
    let results = db.search_videos("S01E01", &folders).unwrap();
    assert_eq!(results.len(), 2);
    assert!(
        results
            .iter()
            .any(|v| v.path == "/media/BreakingBad/S01E01.mp4")
    );
    assert!(
        results
            .iter()
            .any(|v| v.path == "/media/TheWire/S01E01.mp4")
    );
    assert!(
        !results
            .iter()
            .any(|v| v.path == "/media/GameOfThrones/S01E01.mp4")
    );

    // 2. Search with empty folders list (All)
    let all_results = db.search_videos("S01E01", &[]).unwrap();
    assert_eq!(all_results.len(), 3);
}

#[test]
fn test_search_videos_negative_query_grouping() {
    let mut db = Database::open_in_memory().unwrap();
    db.batch_insert_videos(&[
        VideoRecord::new(
            0,
            "Cowboy Bebop - 01 - Asteroid Blues",
            "/media/CowboyBebop/01-AsteroidBlues.mkv",
        ),
        VideoRecord::new(
            0,
            "Cowboy Bebop - 05 - Ballad of Fallen Angels",
            "/media/CowboyBebop/05-BalladOfFallenAngels.mkv",
        ),
        VideoRecord::new(
            0,
            "Eek The Cat - 01 - Misereek",
            "/media/EekTheCat/01-Misereek.mkv",
        ),
        VideoRecord::new(
            0,
            "Trigun - 01 - The $$60 Billion Man",
            "/media/Trigun/01.mkv",
        ),
    ])
    .unwrap();

    // 1. Mixed query: positive (OR) and negative (AND)
    // "cowboy, eek, not fallen" -> (cowboy OR eek) AND (NOT fallen)
    let mixed = db.search_videos("cowboy, eek, not fallen", &[]).unwrap();
    assert_eq!(mixed.len(), 2);
    assert!(mixed.iter().any(|v| v.path.contains("01-AsteroidBlues")));
    assert!(mixed.iter().any(|v| v.path.contains("EekTheCat")));
    assert!(
        !mixed
            .iter()
            .any(|v| v.path.contains("BalladOfFallenAngels"))
    );

    // 2. Multiple negative queries: cowboy, not fallen, !asteroid
    // "cowboy, not fallen, !asteroid" -> (cowboy) AND (NOT fallen AND NOT asteroid)
    let multi_not = db
        .search_videos("cowboy, not fallen, !asteroid", &[])
        .unwrap();
    assert_eq!(multi_not.len(), 0);

    // 3. Pure negative query: not eek, not trigun
    // -> NOT eek AND NOT trigun (should match both Cowboy Bebop episodes)
    let pure_not = db.search_videos("not eek, not trigun", &[]).unwrap();
    assert_eq!(pure_not.len(), 2);
    assert!(pure_not.iter().all(|v| v.path.contains("CowboyBebop")));

    // 4. Case-insensitive NOT and hyphen replacement matching wazoo-js
    let case_and_hyphen = db.search_videos("cowboy, NOT asteroid-blues", &[]).unwrap();
    assert_eq!(case_and_hyphen.len(), 1);
    assert!(case_and_hyphen[0].path.contains("BalladOfFallenAngels"));
}

#[test]
fn test_search_videos_like_wildcard_escaping() {
    let db = Database::open_in_memory().unwrap();
    db.insert_or_update_video(&VideoRecord::new(
        0,
        "Video 100% Real",
        "/media/100%_Real.mkv",
    ))
    .unwrap();
    db.insert_or_update_video(&VideoRecord::new(
        0,
        "Video 1000 Real",
        "/media/1000_Real.mkv",
    ))
    .unwrap();
    db.insert_or_update_video(&VideoRecord::new(
        0,
        "Video 100aReal",
        "/media/100aReal.mkv",
    ))
    .unwrap();

    // Querying "100%" should only match the literal "100%", not "1000" or "100a"
    let res_percent = db.search_videos("100%", &[]).unwrap();
    assert_eq!(res_percent.len(), 1);
    assert_eq!(res_percent[0].path, "/media/100%_Real.mkv");

    // Querying "100_" should only match literal "100_", not "100a" or "100%"
    db.insert_or_update_video(&VideoRecord::new(
        0,
        "Video 100_literal",
        "/media/100_literal.mkv",
    ))
    .unwrap();
    let res_underscore = db.search_videos("100_", &[]).unwrap();
    assert_eq!(res_underscore.len(), 1);
    assert_eq!(res_underscore[0].path, "/media/100_literal.mkv");
}

#[test]
fn test_misc_virtual_folder_crud_and_search() {
    let mut db = Database::open_in_memory().unwrap();
    assert_eq!(db.get_misc_video_count().unwrap(), 0);
    assert!(!db.has_misc_videos().unwrap());

    // 1. Insert normal library video
    db.insert_or_update_video(&VideoRecord::new(
        0,
        "Regular Movie",
        "/media/movies/regular.mkv",
    ))
    .unwrap();

    // 2. Check if file is in other folder
    let media_folders = vec!["/media/movies".to_string()];
    assert!(
        db.is_video_in_other_folder("/media/movies/regular.mkv", &media_folders)
            .unwrap()
    );
    assert!(
        db.is_video_in_other_folder("/media/movies/new_unindexed.mkv", &media_folders)
            .unwrap()
    );
    assert!(
        !db.is_video_in_other_folder("/home/user/Downloads/random.mkv", &media_folders)
            .unwrap()
    );

    // 3. Insert Misc video
    db.insert_misc_video("Random Clip", "/home/user/Downloads/random.mkv")
        .unwrap();
    assert_eq!(db.get_misc_video_count().unwrap(), 1);
    assert!(db.has_misc_videos().unwrap());
    assert_eq!(db.get_video_count().unwrap(), 2);

    // Once in Misc, is_video_in_other_folder returns false (it is in Misc, not an *other* folder)
    assert!(
        !db.is_video_in_other_folder("/home/user/Downloads/random.mkv", &media_folders)
            .unwrap()
    );

    // 4. Search filtering with folder = "Misc"
    let misc_results = db.search_videos("", &["Misc".to_string()]).unwrap();
    assert_eq!(misc_results.len(), 1);
    assert_eq!(misc_results[0].path, "/home/user/Downloads/random.mkv");
    assert_eq!(misc_results[0].folder.as_deref(), Some("Misc"));

    // 5. Search with empty folder (All) returns both regular and Misc videos
    let all_results = db.search_videos("", &[]).unwrap();
    assert_eq!(all_results.len(), 2);

    // 6. Prune simulation: regular video is not in existing_paths, should be pruned.
    // Misc video file doesn't exist on disk, so test prune when file is missing:
    let pruned = db.prune_missing_videos(&[]).unwrap();
    // Both pruned since neither exists on disk
    assert_eq!(pruned, 2);
    assert_eq!(db.get_video_count().unwrap(), 0);

    // 7. Clear misc videos specifically
    db.insert_misc_video("Clip 1", "/path/1.mkv").unwrap();
    db.insert_misc_video("Clip 2", "/path/2.mkv").unwrap();
    db.insert_or_update_video(&VideoRecord::new(0, "Regular", "/media/regular.mkv"))
        .unwrap();
    assert_eq!(db.get_misc_video_count().unwrap(), 2);

    let cleared = db.clear_misc_videos().unwrap();
    assert_eq!(cleared, 2);
    assert_eq!(db.get_misc_video_count().unwrap(), 0);
    assert_eq!(db.get_video_count().unwrap(), 1);
}
