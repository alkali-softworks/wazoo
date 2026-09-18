/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Command-Line Interface Parser
 *
 * Parses startup arguments and options, providing support for search queries,
 * direct video file playback, help information, version display, and argument verification.
 */

use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CliArgs {
    pub query: Option<String>,
    pub file: Option<PathBuf>,
    pub files: Vec<PathBuf>,
}

impl CliArgs {
    pub fn from_query(query: Option<String>) -> Self {
        Self {
            query,
            file: None,
            files: Vec::new(),
        }
    }

    pub fn from_file(file: PathBuf) -> Self {
        Self {
            query: None,
            file: Some(file.clone()),
            files: vec![file],
        }
    }

    pub fn from_files(files: Vec<PathBuf>) -> Self {
        let file = files.first().cloned();
        Self {
            query: None,
            file,
            files,
        }
    }
}

impl From<Option<String>> for CliArgs {
    fn from(query: Option<String>) -> Self {
        Self {
            query,
            file: None,
            files: Vec::new(),
        }
    }
}

pub fn decode_file_url(url: &str) -> String {
    let raw = url.strip_prefix("file://").unwrap_or(url);
    let bytes = raw.as_bytes();
    let mut decoded = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..=i + 2]).unwrap_or(""),
                16,
            ) {
                decoded.push(val);
                i += 3;
                continue;
            }
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(decoded).unwrap_or_else(|_| raw.to_string())
}

pub fn is_video_path(s: &str) -> bool {
    let path = Path::new(s);
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        wazoo_scanner::VIDEO_EXTENSIONS.contains(&ext_lower.as_str())
    } else {
        false
    }
}

pub fn resolve_file_arg(arg: &str) -> Option<PathBuf> {
    let decoded = decode_file_url(arg);
    let p = PathBuf::from(&decoded);
    if p.is_file() {
        return Some(std::fs::canonicalize(&p).unwrap_or(p));
    }
    if is_video_path(&decoded) {
        if let Ok(abs) = std::env::current_dir() {
            let joined = abs.join(&decoded);
            if joined.is_file() {
                return Some(std::fs::canonicalize(&joined).unwrap_or(joined));
            }
            return Some(joined);
        }
        return Some(p);
    }
    None
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
        } else if arg == "--set-default-video" || arg == "--set-default-mkv" {
            match crate::platform::set_as_default_video_player() {
                Ok(()) => {
                    println!("Successfully registered Wazoo as default player for video formats (MKV, MP4, WebM, AVI, MOV).");
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error registering default video player: {e}");
                    std::process::exit(1);
                }
            }
        } else if arg == "-q" || arg == "--query" {
            if let Some(val) = iter.next() {
                query_flag = Some(val);
            } else {
                eprintln!("error: flag '{}' requires a query value", arg);
                eprintln!("Usage: wazoo [OPTIONS] [FILE | QUERY]...\nFor more information, try '--help'.");
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
            eprintln!("Usage: wazoo [OPTIONS] [FILE | QUERY]...\nFor more information, try '--help'.");
            std::process::exit(1);
        } else {
            positional.push(arg);
        }
    }

    let mut files = Vec::new();
    let mut query_positional = Vec::new();

    // Check if the joined positional arguments point to an existing file on disk (e.g. unquoted filename with spaces)
    let joined = positional.join(" ");
    let decoded_joined = decode_file_url(&joined);
    let p_joined = PathBuf::from(&decoded_joined);
    if !positional.is_empty() && p_joined.is_file() {
        files.push(std::fs::canonicalize(&p_joined).unwrap_or(p_joined));
    } else {
        for arg in positional {
            if let Some(resolved) = resolve_file_arg(&arg) {
                files.push(resolved);
            } else {
                query_positional.push(arg);
            }
        }
    }

    let file = files.first().cloned();

    let query = query_flag
        .or_else(|| {
            if query_positional.is_empty() {
                None
            } else {
                Some(query_positional.join(" "))
            }
        })
        .map(|q| q.trim().to_string())
        .filter(|q| !q.is_empty());

    CliArgs {
        query,
        file,
        files,
    }
}

fn print_help_and_exit() -> ! {
    println!(
        "\
Wazoo - Ambient media engine for non-stop viewing

Usage: wazoo [OPTIONS] [FILE | QUERY]...

Arguments:
  [FILE]               Video file to play directly (e.g. 'movie.mkv')
  [QUERY]...           Initial search query to filter videos (e.g. 'wazoo boku')

Options:
  -q, --query <QUERY>  Search query to filter videos
      --set-default-video Register Wazoo as default player for video formats
      --set-default-mkv   Alias for --set-default-video
  -h, --help           Print help
  -V, --version        Print version"
    );
    std::process::exit(0);
}

