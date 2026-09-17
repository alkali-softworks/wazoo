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
}
