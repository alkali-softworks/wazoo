use wazoo_app::cli::parse_cli_args_from;

#[test]
fn test_cli_parsing() {
    assert_eq!(parse_cli_args_from(Vec::<String>::new()).query, None);
    assert_eq!(parse_cli_args_from(vec!["   ".to_string()]).query, None);
    assert_eq!(
        parse_cli_args_from(vec!["boku".to_string()]).query,
        Some("boku".to_string())
    );
    assert_eq!(
        parse_cli_args_from(vec!["boku".to_string(), "hero".to_string()]).query,
        Some("boku hero".to_string())
    );
    assert_eq!(
        parse_cli_args_from(vec!["-q".to_string(), "boku".to_string()]).query,
        Some("boku".to_string())
    );
    assert_eq!(
        parse_cli_args_from(vec!["--query=boku".to_string()]).query,
        Some("boku".to_string())
    );
    assert_eq!(
        parse_cli_args_from(vec![
            "--".to_string(),
            "-special".to_string(),
            "video".to_string()
        ])
        .query,
        Some("-special video".to_string())
    );

    // Video file detection
    let mkv_args = parse_cli_args_from(vec!["/media/test_movie.mkv".to_string()]);
    assert_eq!(mkv_args.query, None);
    assert!(mkv_args.file.is_some());
    assert!(mkv_args.file.unwrap().to_str().unwrap().ends_with("test_movie.mkv"));

    let mp4_args = parse_cli_args_from(vec!["/media/clip.mp4".to_string()]);
    assert!(mp4_args.file.is_some());

    let webm_args = parse_cli_args_from(vec!["/media/clip.webm".to_string()]);
    assert!(webm_args.file.is_some());

    // File URL with spaces percent-encoded
    let url_args = parse_cli_args_from(vec!["file:///media/My%20Video.mkv".to_string()]);
    assert_eq!(url_args.query, None);
    assert!(url_args.file.is_some());
    assert!(url_args.file.unwrap().to_str().unwrap().ends_with("My Video.mkv"));

    // Combined query flag and direct video file
    let combined = parse_cli_args_from(vec![
        "-q".to_string(),
        "ambient".to_string(),
        "/media/film.mkv".to_string(),
    ]);
    assert_eq!(combined.query, Some("ambient".to_string()));
    assert!(combined.file.is_some());
    assert!(combined.file.unwrap().to_str().unwrap().ends_with("film.mkv"));

    // Multiple video files
    let multi = parse_cli_args_from(vec![
        "/media/part1.mkv".to_string(),
        "/media/part2.mkv".to_string(),
    ]);
    assert_eq!(multi.files.len(), 2);
    assert!(multi.file.as_ref().unwrap().to_string_lossy().replace('\\', "/").ends_with("/media/part1.mkv"));
    assert!(multi.files[1].to_string_lossy().replace('\\', "/").ends_with("/media/part2.mkv"));

    // Foreground / no-detach flags
    let default_bg = parse_cli_args_from(vec!["boku".to_string()]);
    assert!(!default_bg.foreground);

    let fg_short = parse_cli_args_from(vec!["-f".to_string(), "boku".to_string()]);
    assert!(fg_short.foreground);
    assert_eq!(fg_short.query, Some("boku".to_string()));

    let fg_long = parse_cli_args_from(vec!["--foreground".to_string(), "/media/film.mkv".to_string()]);
    assert!(fg_long.foreground);
    assert!(fg_long.file.is_some());

    let fg_nodetach = parse_cli_args_from(vec!["--no-detach".to_string()]);
    assert!(fg_nodetach.foreground);
}
