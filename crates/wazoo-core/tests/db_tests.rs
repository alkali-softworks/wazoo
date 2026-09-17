use wazoo_core::{Database, VideoRecord};

#[test]
fn test_db_operations() {
    let db = Database::open_in_memory().unwrap();
    assert_eq!(db.get_video_count().unwrap(), 0);

    let video = VideoRecord {
        id: 0,
        name: "Ambient Video 1".to_string(),
        path: "/media/ambient1.mp4".to_string(),
    };

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
        VideoRecord {
            id: 0,
            name: "Vid 1".to_string(),
            path: "/home/user/media/folder_a/1.mp4".to_string(),
        },
        VideoRecord {
            id: 0,
            name: "Vid 2".to_string(),
            path: "/home/user/media/folder_a/sub/2.mp4".to_string(),
        },
        VideoRecord {
            id: 0,
            name: "Vid 3".to_string(),
            path: "/home/user/media/folder_b/3.mp4".to_string(),
        },
        VideoRecord {
            id: 0,
            name: "Vid 4".to_string(),
            path: "/home/user/media/folder_a_other/4.mp4".to_string(),
        },
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
        VideoRecord {
            id: 0,
            name: "Breaking Bad S01E01".to_string(),
            path: "/media/BreakingBad/S01E01.mp4".to_string(),
        },
        VideoRecord {
            id: 0,
            name: "Breaking Bad S01E02".to_string(),
            path: "/media/BreakingBad/S01E02.mp4".to_string(),
        },
        VideoRecord {
            id: 0,
            name: "Game of Thrones S01E01".to_string(),
            path: "/media/GameOfThrones/S01E01.mp4".to_string(),
        },
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
        VideoRecord {
            id: 0,
            name: "Breaking Bad S01E01".to_string(),
            path: "/media/BreakingBad/S01E01.mp4".to_string(),
        },
        VideoRecord {
            id: 0,
            name: "Game of Thrones S01E01".to_string(),
            path: "/media/GameOfThrones/S01E01.mp4".to_string(),
        },
        VideoRecord {
            id: 0,
            name: "The Wire S01E01".to_string(),
            path: "/media/TheWire/S01E01.mp4".to_string(),
        },
    ])
    .unwrap();

    // 1. Search with multiple folders: BreakingBad and TheWire (omit GameOfThrones)
    let folders = vec![
        "/media/BreakingBad".to_string(),
        "/media/TheWire".to_string(),
    ];
    let results = db.search_videos("S01E01", &folders).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|v| v.path == "/media/BreakingBad/S01E01.mp4"));
    assert!(results.iter().any(|v| v.path == "/media/TheWire/S01E01.mp4"));
    assert!(!results.iter().any(|v| v.path == "/media/GameOfThrones/S01E01.mp4"));

    // 2. Search with empty folders list (All)
    let all_results = db.search_videos("S01E01", &[]).unwrap();
    assert_eq!(all_results.len(), 3);
}

#[test]
fn test_search_videos_negative_query_grouping() {
    let mut db = Database::open_in_memory().unwrap();
    db.batch_insert_videos(&[
        VideoRecord {
            id: 0,
            name: "Cowboy Bebop - 01 - Asteroid Blues".to_string(),
            path: "/media/CowboyBebop/01-AsteroidBlues.mkv".to_string(),
        },
        VideoRecord {
            id: 0,
            name: "Cowboy Bebop - 05 - Ballad of Fallen Angels".to_string(),
            path: "/media/CowboyBebop/05-BalladOfFallenAngels.mkv".to_string(),
        },
        VideoRecord {
            id: 0,
            name: "Eek The Cat - 01 - Misereek".to_string(),
            path: "/media/EekTheCat/01-Misereek.mkv".to_string(),
        },
        VideoRecord {
            id: 0,
            name: "Trigun - 01 - The $$60 Billion Man".to_string(),
            path: "/media/Trigun/01.mkv".to_string(),
        },
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
    db.insert_or_update_video(&VideoRecord {
        id: 0,
        name: "Video 100% Real".to_string(),
        path: "/media/100%_Real.mkv".to_string(),
    })
    .unwrap();
    db.insert_or_update_video(&VideoRecord {
        id: 0,
        name: "Video 1000 Real".to_string(),
        path: "/media/1000_Real.mkv".to_string(),
    })
    .unwrap();
    db.insert_or_update_video(&VideoRecord {
        id: 0,
        name: "Video 100aReal".to_string(),
        path: "/media/100aReal.mkv".to_string(),
    })
    .unwrap();

    // Querying "100%" should only match the literal "100%", not "1000" or "100a"
    let res_percent = db.search_videos("100%", &[]).unwrap();
    assert_eq!(res_percent.len(), 1);
    assert_eq!(res_percent[0].path, "/media/100%_Real.mkv");

    // Querying "100_" should only match literal "100_", not "100a" or "100%"
    db.insert_or_update_video(&VideoRecord {
        id: 0,
        name: "Video 100_literal".to_string(),
        path: "/media/100_literal.mkv".to_string(),
    })
    .unwrap();
    let res_underscore = db.search_videos("100_", &[]).unwrap();
    assert_eq!(res_underscore.len(), 1);
    assert_eq!(res_underscore[0].path, "/media/100_literal.mkv");
}
