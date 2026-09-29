use wazoo_app::format::{
    clean_name, folder_basename, format_descriptive_title, format_title_lines, format_video_title,
};

#[test]
fn test_clean_name() {
    assert_eq!(
        clean_name("The.Matrix.1999.1080p.x264-VXT"),
        "The Matrix 1999"
    );
    assert_eq!(
        clean_name("[SubsPlease] Frieren - 01 (1080p) [HEVC]"),
        "Frieren - 01"
    );
}

#[test]
fn test_format_video_title() {
    assert_eq!(
        format_video_title("/media/Movies/Interstellar.2014.1080p.mkv"),
        "Interstellar 2014"
    );
}

#[test]
fn test_format_descriptive_title_deduplication() {
    // When the title already incorporates the series/folder name, don't duplicate it
    assert_eq!(
        format_descriptive_title(
            "/media/Anime/Last Exile Gin`yoku no Fam/Last Exile - Gin`yoku no Fam - 08.mkv"
        ),
        "Last Exile - Gin'yoku no Fam — Episode 08"
    );

    // When the file name is just an episode number, prepend the folder name
    assert_eq!(
        format_descriptive_title("/media/Anime/Cowboy Bebop/01.mkv"),
        "Cowboy Bebop — Episode 01"
    );

    // Generic folders like Movies should not be prepended
    assert_eq!(
        format_descriptive_title("/media/Movies/Interstellar.2014.1080p.mkv"),
        "Interstellar 2014"
    );
}

#[test]
fn test_format_title_lines() {
    assert_eq!(
        format_title_lines("/media/Anime/Cowboy Bebop/01.mkv"),
        ("Cowboy Bebop".to_string(), Some("Episode 01".to_string()))
    );
    assert_eq!(
        format_title_lines("/media/Anime/Cowboy Bebop/01 - Asteroid Blues.mkv"),
        (
            "Cowboy Bebop".to_string(),
            Some("Episode 01 - Asteroid Blues".to_string())
        )
    );
    assert_eq!(
        format_title_lines("/media/Movies/Interstellar.2014.1080p.mkv"),
        ("Interstellar 2014".to_string(), None)
    );
    assert_eq!(
        format_title_lines("/media/Movies/The Lord of the Rings - The Fellowship of the Ring.mkv"),
        (
            "The Lord of the Rings".to_string(),
            Some("The Fellowship of the Ring".to_string())
        )
    );
}

#[test]
fn test_folder_basename() {
    assert_eq!(folder_basename("/media/Movies"), "Movies");
    assert_eq!(folder_basename("/media/Movies/"), "Movies");
    assert_eq!(folder_basename("C:\\Media\\Anime\\"), "Anime");
    assert_eq!(folder_basename("Anime"), "Anime");
}

#[test]
fn test_season_episode_recognition() {
    assert_eq!(
        format_title_lines("Kare Kano s01e18 Progress.mkv"),
        (
            "Kare Kano".to_string(),
            Some("S01E18 - Progress".to_string())
        )
    );
    assert_eq!(
        format_descriptive_title("Kare Kano s01e18 Progress.mkv"),
        "Kare Kano — S01E18 - Progress"
    );
    assert_eq!(
        format_descriptive_title("/media/Anime/Kare Kano/s01e18 Progress.mkv"),
        "Kare Kano — S01E18 - Progress"
    );
    assert_eq!(
        format_descriptive_title("Kare Kano s01e18.mkv"),
        "Kare Kano — S01E18"
    );
}
