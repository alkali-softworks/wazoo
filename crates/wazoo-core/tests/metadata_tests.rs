use wazoo_core::{Database, MediaMetadata, VideoRecord};

#[test]
fn test_metadata_crud() {
    let db = Database::open_in_memory().expect("open db");

    let video = VideoRecord::new(1, "The Matrix", "/media/movies/The Matrix (1999).mkv");
    let video_id = db.insert_or_update_video(&video).expect("insert video");

    let meta = MediaMetadata {
        video_id,
        plex_rating_key: Some("101".to_string()),
        title: "The Matrix".to_string(),
        series_title: None,
        season_number: None,
        episode_number: None,
        year: Some(1999),
        tagline: Some("Welcome to the Real World".to_string()),
        summary: Some("A computer hacker learns from mysterious rebels about the true nature of his reality.".to_string()),
        content_rating: Some("R".to_string()),
        duration_ms: Some(8160000),
        rating: Some(8.7),
        studio: Some("Warner Bros.".to_string()),
        genres: vec!["Action".to_string(), "Sci-Fi".to_string()],
        directors: vec!["Lana Wachowski".to_string(), "Lilly Wachowski".to_string()],
        actors: vec!["Keanu Reeves".to_string(), "Laurence Fishburne".to_string()],
        thumb_url: Some("/library/metadata/101/thumb/123".to_string()),
        art_url: Some("/library/metadata/101/art/123".to_string()),
        updated_at: Some(1600000000),
    };

    db.upsert_metadata(&meta).expect("upsert metadata");

    assert_eq!(db.get_metadata_count().expect("count"), 1);

    let retrieved = db
        .get_metadata_by_video_id(video_id)
        .expect("query")
        .expect("exists");
    assert_eq!(retrieved.title, "The Matrix");
    assert_eq!(retrieved.year, Some(1999));
    assert_eq!(retrieved.genres, vec!["Action", "Sci-Fi"]);
    assert_eq!(retrieved.directors.len(), 2);
    assert_eq!(retrieved.content_rating.as_deref(), Some("R"));

    let by_path = db
        .get_metadata_by_path("/media/movies/The Matrix (1999).mkv")
        .expect("query path")
        .expect("exists");
    assert_eq!(by_path.video_id, video_id);
}

#[test]
fn test_search_with_metadata() {
    let db = Database::open_in_memory().expect("open db");

    let v1 = VideoRecord::new(1, "matrix_rel", "/media/movies/m1.mkv");
    let id1 = db.insert_or_update_video(&v1).expect("insert 1");
    db.upsert_metadata(&MediaMetadata {
        video_id: id1,
        plex_rating_key: Some("1".to_string()),
        title: "The Matrix".to_string(),
        year: Some(1999),
        genres: vec!["Sci-Fi".to_string(), "Action".to_string()],
        directors: vec!["Wachowski".to_string()],
        ..Default::default()
    })
    .expect("meta 1");

    let v2 = VideoRecord::new(2, "pulp_fic", "/media/movies/p2.mkv");
    let id2 = db.insert_or_update_video(&v2).expect("insert 2");
    db.upsert_metadata(&MediaMetadata {
        video_id: id2,
        plex_rating_key: Some("2".to_string()),
        title: "Pulp Fiction".to_string(),
        year: Some(1994),
        genres: vec!["Crime".to_string(), "Drama".to_string()],
        directors: vec!["Quentin Tarantino".to_string()],
        ..Default::default()
    })
    .expect("meta 2");

    // Search by genre: "Sci-Fi"
    let res = db.search_videos("Sci-Fi", &[]).expect("search Sci-Fi");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, id1);

    // Search by year: "1994"
    let res = db.search_videos("1994", &[]).expect("search 1994");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, id2);

    // Search by director: "tarantino"
    let res = db.search_videos("tarantino", &[]).expect("search tarantino");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, id2);

    // Negative search: "movies, not crime"
    let res = db.search_videos("movies, not crime", &[]).expect("search neg");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, id1);
}

