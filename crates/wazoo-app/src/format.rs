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
        Regex::new(r"(?i)^Disc\s+\d+").unwrap(),
        Regex::new(r"(?i)^Vol(ume)?\s+\d+").unwrap(),
        Regex::new(r"^\d+$").unwrap(),
    ]
});

static ROOT_CATEGORY_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)^Movies?$").unwrap(),
        Regex::new(r"(?i)^Films?$").unwrap(),
        Regex::new(r"(?i)^Videos?$").unwrap(),
        Regex::new(r"(?i)^Media$").unwrap(),
        Regex::new(r"(?i)^Downloads?$").unwrap(),
        Regex::new(r"(?i)^Desktop$").unwrap(),
        Regex::new(r"(?i)^Documents?$").unwrap(),
        Regex::new(r"(?i)^Home$").unwrap(),
        Regex::new(r"(?i)^Mnt$").unwrap(),
        Regex::new(r"(?i)^Users?$").unwrap(),
        Regex::new(r"(?i)^[a-z]$").unwrap(),
        Regex::new(r"(?i)^Anime$").unwrap(),
        Regex::new(r"(?i)^TV(\s*Shows?)?$").unwrap(),
        Regex::new(r"(?i)^Series$").unwrap(),
        Regex::new(r"(?i)^Cartoons?$").unwrap(),
    ]
});

static RE_EPISODE_NUM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(?:EP?|Episode)\s*(\d+)$").unwrap());

static RE_SEASON_EPISODE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(?:S|Season\s*)(\d{1,2})\s*(?:E|EP|Episode\s*)(\d{1,4}(?:(?:-[eE]?|[eE])\d{1,4})?)\b",
    )
    .unwrap()
});

static RE_X_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(\d{1,2})x(\d{1,4})\b").unwrap());

static RE_SPECIAL_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(?:OVA|Special|SP)\b").unwrap());

static RE_LEADING_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(\d{1,3})\s*[-—:]\s*(.+)$").unwrap());

static RE_TRAILING_EPISODE_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\s+((?:S\d+\s*)?(?:E|EP|Episode)\s*\d{1,3}|S\d+\s*E\d+|OVA\s*\d*)$").unwrap()
});

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SeasonEpisodeMatch<'a> {
    pub prefix: &'a str,
    pub marker: String,
    pub suffix: &'a str,
}

pub fn find_season_episode_match(s: &str) -> Option<SeasonEpisodeMatch<'_>> {
    if let Some(caps) = RE_SEASON_EPISODE.captures(s) {
        let mat = caps.get(0).unwrap();
        let s_num: u32 = caps[1].parse().unwrap_or(1);
        let ep_part = &caps[2];
        let marker = if ep_part.chars().all(|c| c.is_ascii_digit()) {
            let e_num: u32 = ep_part.parse().unwrap_or(1);
            if ep_part.len() >= 3 {
                format!("S{s_num:02}E{ep_part}")
            } else {
                format!("S{s_num:02}E{e_num:02}")
            }
        } else {
            let upper = ep_part.to_ascii_uppercase();
            format!("S{:02}E{}", s_num, upper.trim_start_matches('E'))
        };

        let raw_prefix = &s[..mat.start()];
        let raw_suffix = &s[mat.end()..];

        let prefix = raw_prefix
            .trim()
            .trim_matches(|c: char| {
                c == '-' || c == '—' || c == '–' || c == ':' || c == '.' || c == ' '
            })
            .trim();
        let suffix = raw_suffix
            .trim()
            .trim_matches(|c: char| {
                c == '-' || c == '—' || c == '–' || c == ':' || c == '.' || c == ' '
            })
            .trim();

        return Some(SeasonEpisodeMatch {
            prefix,
            marker,
            suffix,
        });
    }

    if let Some(caps) = RE_X_EPISODE.captures(s) {
        let mat = caps.get(0).unwrap();
        let s_num: u32 = caps[1].parse().unwrap_or(1);
        let e_num: u32 = caps[2].parse().unwrap_or(1);
        let marker = format!("S{s_num:02}E{e_num:02}");

        let raw_prefix = &s[..mat.start()];
        let raw_suffix = &s[mat.end()..];

        let prefix = raw_prefix
            .trim()
            .trim_matches(|c: char| {
                c == '-' || c == '—' || c == '–' || c == ':' || c == '.' || c == ' '
            })
            .trim();
        let suffix = raw_suffix
            .trim()
            .trim_matches(|c: char| {
                c == '-' || c == '—' || c == '–' || c == ':' || c == '.' || c == ' '
            })
            .trim();

        // Avoid false positive on titles starting with NxN (e.g. "3x3 Eyes") when no preceding show name
        if !prefix.is_empty() {
            return Some(SeasonEpisodeMatch {
                prefix,
                marker,
                suffix,
            });
        }
    }

    None
}

/// Cleans media filenames by removing tags, codecs, resolution, brackets, etc.
pub fn clean_name(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }

    // Normalize backticks and acute accents into standard apostrophes (e.g. Gin`yoku -> Gin'yoku)
    let with_apostrophes = name.replace(['`', '´'], "'");

    // Replace dots and underscores with spaces
    let mut cleaned = with_apostrophes.replace(['.', '_'], " ");

    // Remove square brackets content
    cleaned = RE_SQUARE_BRACKETS.replace_all(&cleaned, "").to_string();

    // Keep parentheses ONLY if they contain a 4-digit year (e.g., (1994)) or season/episode marker
    cleaned = RE_PARENS
        .replace_all(&cleaned, |caps: &regex::Captures| {
            let inner = &caps[1];
            if inner.len() == 4 && inner.chars().all(|c| c.is_ascii_digit()) {
                format!("({inner})")
            } else if RE_SEASON_EPISODE.is_match(inner) || RE_X_EPISODE.is_match(inner) {
                format!(" {inner} ")
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

pub fn is_category_or_root_folder(name: &str) -> bool {
    let n = name.trim();
    ROOT_CATEGORY_REGEXES.iter().any(|r| r.is_match(n))
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

    // If folder is a season/specials subfolder (e.g. "Season 1"), step back to parent series folder
    while is_generic_folder(folder_name) && folder_idx > 0 {
        folder_idx -= 1;
        folder_name = segments[folder_idx];
    }

    if is_generic_folder(folder_name)
        || is_category_or_root_folder(folder_name)
        || folder_name.contains(':')
        || folder_name == "/"
        || folder_name.is_empty()
    {
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

fn normalize_for_comparison(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Checks whether a video title already incorporates the series/folder name,
/// preventing redundant "Series — Series - 01" duplications.
pub fn is_title_redundant_with_folder(folder: &str, title: &str) -> bool {
    let norm_folder = normalize_for_comparison(folder);
    let norm_title = normalize_for_comparison(title);

    if norm_folder.is_empty() || norm_title.is_empty() {
        return false;
    }

    norm_title.starts_with(&norm_folder)
        || norm_folder.starts_with(&norm_title)
        || (norm_folder.len() >= 6 && norm_title.contains(&norm_folder))
}

fn is_episode_part(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    // 1-3 digits: e.g. "01", "08", "1", "124" (excluding 4-digit years like 2014)
    if trimmed.chars().all(|c| c.is_ascii_digit()) && trimmed.len() <= 3 {
        return true;
    }
    if RE_EPISODE_NUM.is_match(trimmed) {
        return true;
    }
    if RE_SEASON_EPISODE.is_match(trimmed) || RE_X_EPISODE.is_match(trimmed) {
        return true;
    }
    if RE_SPECIAL_START.is_match(trimmed) {
        return true;
    }
    false
}

fn format_episode_str(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.chars().all(|c| c.is_ascii_digit()) && trimmed.len() <= 3 {
        format!("Episode {trimmed}")
    } else if let Some(caps) = RE_EPISODE_NUM.captures(trimmed) {
        format!("Episode {}", &caps[1])
    } else if let Some(caps) = RE_LEADING_EPISODE.captures(trimmed) {
        format!("Episode {} - {}", &caps[1], &caps[2])
    } else if let Some(se_match) = find_season_episode_match(trimmed) {
        if se_match.prefix.is_empty() {
            if se_match.suffix.is_empty() {
                se_match.marker
            } else {
                format!("{} - {}", se_match.marker, se_match.suffix)
            }
        } else if se_match.suffix.is_empty() {
            format!("{} - {}", se_match.prefix, se_match.marker)
        } else {
            format!(
                "{} - {} - {}",
                se_match.prefix, se_match.marker, se_match.suffix
            )
        }
    } else {
        trimmed.to_string()
    }
}

/// Splits a video path into a primary title (Line 1) and optional subtitle/episode (Line 2).
/// Suitable for clean multi-line display in player overlays and headers.
pub fn format_title_lines(path: &str) -> (String, Option<String>) {
    let title = format_video_title(path);
    let folder = format_video_folder(path);

    // 1. Check if the title contains a season/episode marker (e.g. "Kare Kano s01e18 Progress")
    if let Some(se_match) = find_season_episode_match(&title) {
        let ep_str = if se_match.suffix.is_empty() {
            se_match.marker
        } else {
            format!("{} - {}", se_match.marker, se_match.suffix)
        };

        if !se_match.prefix.is_empty() {
            let show_name =
                if !folder.is_empty() && is_title_redundant_with_folder(&folder, se_match.prefix) {
                    folder
                } else {
                    se_match.prefix.to_string()
                };
            return (show_name, Some(ep_str));
        } else if !folder.is_empty()
            && !is_generic_folder(&folder)
            && !is_category_or_root_folder(&folder)
        {
            return (folder, Some(ep_str));
        } else {
            return (ep_str, None);
        }
    }

    // 2. Folder check when folder is distinct from title
    let has_distinct_folder = !folder.is_empty()
        && !folder.eq_ignore_ascii_case(&title)
        && !is_title_redundant_with_folder(&folder, &title);

    if has_distinct_folder {
        return (folder, Some(format_episode_str(&title)));
    }

    // 3. When folder is absent or redundant with title, inspect the title structure
    let parts: Vec<&str> = title
        .split(|c| c == '—' || c == '–')
        .flat_map(|s| s.split(" - "))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() >= 2 {
        if let Some(ep_idx) = (1..parts.len()).find(|&i| is_episode_part(parts[i])) {
            let primary = parts[..ep_idx].join(" - ");
            let mut ep_tokens = Vec::new();
            ep_tokens.push(format_episode_str(parts[ep_idx]));
            for &rem in &parts[ep_idx + 1..] {
                ep_tokens.push(rem.to_string());
            }
            let secondary = ep_tokens.join(" - ");
            return (primary, Some(secondary));
        }

        // Subtitle split (e.g. "Main Title - Subtitle")
        if parts.len() == 2 {
            return (parts[0].to_string(), Some(parts[1].to_string()));
        }
    }

    // 4. Check for space-separated trailing episode markers (e.g. "Series S01E08" or "Series 08")
    if let Some(mat) = RE_TRAILING_EPISODE_MARKER.find(&title) {
        let prefix = title[..mat.start()].trim();
        let marker = title[mat.start()..].trim();
        if !prefix.is_empty()
            && (is_episode_part(marker)
                || (!folder.is_empty() && is_title_redundant_with_folder(&folder, prefix)))
        {
            return (prefix.to_string(), Some(format_episode_str(marker)));
        }
    }

    (title, None)
}

pub fn format_descriptive_title(path: &str) -> String {
    let (primary, secondary) = format_title_lines(path);
    if let Some(sec) = secondary {
        format!("{primary} — {sec}")
    } else {
        primary
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
