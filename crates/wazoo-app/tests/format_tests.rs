use wazoo_app::format::{clean_name, folder_basename, format_video_title, ucwords};

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
fn test_folder_basename() {
    assert_eq!(folder_basename("/media/Movies"), "Movies");
    assert_eq!(folder_basename("/media/Movies/"), "Movies");
    assert_eq!(folder_basename("C:\\Media\\Anime\\"), "Anime");
    assert_eq!(folder_basename("Anime"), "Anime");
}

#[test]
fn test_ucwords() {
    assert_eq!(ucwords("anime"), "Anime");
    assert_eq!(ucwords("movies"), "Movies");
    assert_eq!(ucwords("tv"), "Tv");
    assert_eq!(ucwords("sat_morning_shows"), "Sat_Morning_Shows");
    assert_eq!(ucwords("Sat_Morning_Shows"), "Sat_Morning_Shows");
    assert_eq!(ucwords("my favorite videos"), "My Favorite Videos");
    assert_eq!(ucwords("action-packed"), "Action-Packed");
    assert_eq!(ucwords(""), "");
}

