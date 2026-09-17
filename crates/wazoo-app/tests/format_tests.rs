use wazoo_app::format::{clean_name, folder_basename, format_video_title};

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
