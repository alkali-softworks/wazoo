/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Command-Line Interface Parser
 * 
 * Parses startup arguments and options, providing support for search queries,
 * query flags, help information, version display, and argument verification.
 */

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CliArgs {
    pub query: Option<String>,
}

pub fn parse_cli_args() -> CliArgs {
    parse_cli_args_from(std::env::args().skip(1))
}

pub fn parse_cli_args_from<I>(args: I) -> CliArgs
where
    I: IntoIterator<Item = String>,
{
    let mut positional: Vec<String> = Vec::new();
    let mut query_flag: Option<String> = None;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        if arg == "--help" || arg == "-h" {
            print_help_and_exit();
        } else if arg == "--version" || arg == "-V" {
            println!("wazoo {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        } else if arg == "-q" || arg == "--query" {
            if let Some(val) = iter.next() {
                query_flag = Some(val);
            } else {
                eprintln!("error: flag '{}' requires a query value", arg);
                eprintln!("Usage: wazoo [OPTIONS] [QUERY]...\nFor more information, try '--help'.");
                std::process::exit(1);
            }
        } else if let Some(val) = arg.strip_prefix("--query=") {
            query_flag = Some(val.to_string());
        } else if let Some(val) = arg.strip_prefix("-q=") {
            query_flag = Some(val.to_string());
        } else if arg == "--" {
            for remaining in iter.by_ref() {
                positional.push(remaining);
            }
            break;
        } else if arg.starts_with('-') && arg.len() > 1 {
            eprintln!("error: unexpected argument '{}'", arg);
            eprintln!("Usage: wazoo [OPTIONS] [QUERY]...\nFor more information, try '--help'.");
            std::process::exit(1);
        } else {
            positional.push(arg);
        }
    }

    let query = query_flag
        .or_else(|| {
            if positional.is_empty() {
                None
            } else {
                Some(positional.join(" "))
            }
        })
        .map(|q| q.trim().to_string())
        .filter(|q| !q.is_empty());

    CliArgs { query }
}

fn print_help_and_exit() -> ! {
    println!(
        "\
Wazoo - Ambient media engine for non-stop viewing

Usage: wazoo [OPTIONS] [QUERY]...

Arguments:
  [QUERY]...  Initial search query to filter videos (e.g. 'wazoo boku')

Options:
  -q, --query <QUERY>  Search query to filter videos
  -h, --help           Print help
  -V, --version        Print version"
    );
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        assert_eq!(parse_cli_args_from(Vec::<String>::new()).query, None);
        assert_eq!(parse_cli_args_from(vec!["   ".to_string()]).query, None);
        assert_eq!(parse_cli_args_from(vec!["boku".to_string()]).query, Some("boku".to_string()));
        assert_eq!(parse_cli_args_from(vec!["boku".to_string(), "hero".to_string()]).query, Some("boku hero".to_string()));
        assert_eq!(parse_cli_args_from(vec!["-q".to_string(), "boku".to_string()]).query, Some("boku".to_string()));
        assert_eq!(parse_cli_args_from(vec!["--query=boku".to_string()]).query, Some("boku".to_string()));
        assert_eq!(parse_cli_args_from(vec!["--".to_string(), "-special".to_string(), "video".to_string()]).query, Some("-special video".to_string()));
    }
}
