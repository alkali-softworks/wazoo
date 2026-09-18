/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * String & Time Formatting Utilities
 *
 * Sanitizes and cleans video file titles, extracts folder names, formats timestamps
 * (HH:MM:SS / MM:SS), and produces descriptive media labels.
 */

use regex::Regex;
use std::sync::LazyLock;

static RE_SQUARE_BRACKETS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[[^\]]*\]").unwrap());
static RE_PARENS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(([^)]*)\)").unwrap());
static RE_SEASON_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\(S\d+\)").unwrap());
static RE_MULTIPLE_SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
static RE_TRAILING_GROUP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-[A-Z0-9]{2,}$").unwrap());

static METADATA_TAGS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)dvdrip").unwrap(),
        Regex::new(r"(?i)bdrip").unwrap(),
        Regex::new(r"(?i)bluray").unwrap(),
        Regex::new(r"(?i)webrip").unwrap(),
        Regex::new(r"(?i)hdrip").unwrap(),
        Regex::new(r"(?i)x264").unwrap(),
        Regex::new(r"(?i)x265").unwrap(),
        Regex::new(r"(?i)h264").unwrap(),
        Regex::new(r"(?i)h265").unwrap(),
        Regex::new(r"(?i)hevc").unwrap(),
        Regex::new(r"(?i)1080p").unwrap(),
        Regex::new(r"(?i)720p").unwrap(),
        Regex::new(r"(?i)480p").unwrap(),
        Regex::new(r"(?i)2160p").unwrap(),
        Regex::new(r"(?i)4k").unwrap(),
        Regex::new(r"(?i)aac").unwrap(),
        Regex::new(r"(?i)dts").unwrap(),
        Regex::new(r"(?i)ac3").unwrap(),
        Regex::new(r"(?i)flac").unwrap(),
        Regex::new(r"(?i)complete").unwrap(),
    ]
});

static GENERIC_FOLDER_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)^Season\s+\d+").unwrap(),
        Regex::new(r"(?i)^Series\s+\d+").unwrap(),
        Regex::new(r"(?i)^S\d+$").unwrap(),
        Regex::new(r"(?i)^OVA\s+\d+").unwrap(),
        Regex::new(r"(?i)^Specials?$").unwrap(),
        Regex::new(r"(?i)^Extras?$").unwrap(),
        Regex::new(r"(?i)^Movies?$").unwrap(),
        Regex::new(r"(?i)^Disc\s+\d+").unwrap(),
        Regex::new(r"(?i)^Vol(ume)?\s+\d+").unwrap(),
        Regex::new(r"^\d+$").unwrap(),
    ]
});

/// Cleans media filenames by removing tags, codecs, resolution, brackets, etc.
pub fn clean_name(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }

    // Replace dots and underscores with spaces
    let mut cleaned = name.replace(['.', '_'], " ");

    // Remove square brackets content
    cleaned = RE_SQUARE_BRACKETS.replace_all(&cleaned, "").to_string();

    // Keep parentheses ONLY if they contain a 4-digit year (e.g., (1994))
    cleaned = RE_PARENS
        .replace_all(&cleaned, |caps: &regex::Captures| {
            let inner = &caps[1];
            if inner.len() == 4 && inner.chars().all(|c| c.is_ascii_digit()) {
                format!("({inner})")
            } else {
                String::new()
            }
        })
        .to_string();

    // Remove metadata tags
    for tag in METADATA_TAGS.iter() {
        cleaned = tag.replace_all(&cleaned, "").to_string();
    }

    // Collapse multiple spaces
    cleaned = RE_MULTIPLE_SPACES.replace_all(&cleaned, " ").to_string();

    // Remove trailing release group names (e.g. -VXT)
    cleaned = RE_TRAILING_GROUP.replace_all(&cleaned, "").to_string();

    // Trim leading/trailing punctuation and whitespace
    let trimmed = cleaned.trim();
    trimmed
        .trim_matches(|c: char| c == '-' || c == '.' || c == ' ' || c == ':')
        .to_string()
}

pub fn is_generic_folder(name: &str) -> bool {
    let n = name.trim();
    GENERIC_FOLDER_REGEXES.iter().any(|r| r.is_match(n))
}

pub fn format_video_folder(path: &str) -> String {
    let clean_path = path.strip_prefix("file://").unwrap_or(path);
    let segments: Vec<&str> = clean_path
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .collect();

    if segments.len() < 2 {
        return String::new();
    }

    let mut folder_idx = segments.len() - 2;
    let mut folder_name = segments[folder_idx];

    if is_generic_folder(folder_name) && folder_idx > 0 {
        folder_idx -= 1;
        folder_name = segments[folder_idx];
    }

    if folder_name.contains(':') || folder_name == "/" || folder_name.is_empty() {
        return String::new();
    }

    clean_name(folder_name)
}

pub fn format_video_title(path: &str) -> String {
    let clean_path = path.strip_prefix("file://").unwrap_or(path);
    let file_name = clean_path.split(['/', '\\']).next_back().unwrap_or("");

    // Remove file extension
    let without_ext = match file_name.rfind('.') {
        Some(pos) => &file_name[..pos],
        None => file_name,
    };

    // Remove (S01) etc
    let without_ep = RE_SEASON_PATTERN.replace_all(without_ext, "");

    let cleaned = clean_name(&without_ep);
    if cleaned.is_empty() {
        file_name.to_string()
    } else {
        cleaned
    }
}

pub fn format_descriptive_title(path: &str) -> String {
    let title = format_video_title(path);
    let folder = format_video_folder(path);

    if !folder.is_empty() && !folder.eq_ignore_ascii_case(&title) {
        if title.to_lowercase().starts_with(&folder.to_lowercase()) {
            return title;
        }
        format!("{folder} — {title}")
    } else {
        title
    }
}

pub fn format_time_str(seconds: f64) -> String {
    if seconds <= 0.0 || seconds.is_nan() {
        return "00:00".to_string();
    }
    let total_secs = seconds.round() as u64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{mins:02}:{secs:02}")
}

/// Formats an unsigned integer with digit grouping separators (e.g. 1000 -> "1,000").
pub fn format_number(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + len / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

/// Extracts the trailing folder name from a file or directory path (e.g. "/media/Anime" -> "Anime").
pub fn folder_basename(path: &str) -> &str {
    let clean = path.strip_prefix("file://").unwrap_or(path);
    clean
        .split(['/', '\\'])
        .rfind(|s| !s.is_empty())
        .unwrap_or(clean)
}

/// Capitalizes the first character of each word in a string (e.g. "anime" -> "Anime", "sat morning shows" -> "Sat Morning Shows").
pub fn ucwords(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;

    for c in s.chars() {
        if c.is_alphabetic() {
            if capitalize_next {
                for upper in c.to_uppercase() {
                    result.push(upper);
                }
                capitalize_next = false;
            } else {
                result.push(c);
            }
        } else {
            capitalize_next = true;
            result.push(c);
        }
    }

    result
}


