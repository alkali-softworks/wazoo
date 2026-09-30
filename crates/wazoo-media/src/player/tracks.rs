/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Media Player Tracks & Language Representations
 */

pub type PlayerId = usize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StartTime {
    Beginning,
    Seconds(f64),
    Percent(f64),
    Random,
}

impl From<Option<f64>> for StartTime {
    fn from(opt: Option<f64>) -> Self {
        match opt {
            Some(s) => StartTime::Seconds(s),
            None => StartTime::Beginning,
        }
    }
}

impl From<f64> for StartTime {
    fn from(s: f64) -> Self {
        StartTime::Seconds(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioTrack {
    pub id: i64,
    pub title: Option<String>,
    pub lang: Option<String>,
    pub codec: Option<String>,
    pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubtitleTrack {
    pub id: i64,
    pub title: Option<String>,
    pub lang: Option<String>,
    pub codec: Option<String>,
    pub is_selected: bool,
    pub external_filename: Option<String>,
    pub ff_index: Option<i64>,
}

pub fn language_display_name(code: &str) -> &'static str {
    let lower = code.trim().to_ascii_lowercase();
    match lower.as_str() {
        "en" | "eng" => "English",
        "ja" | "jpn" => "Japanese",
        "es" | "spa" => "Spanish",
        "fr" | "fra" | "fre" => "French",
        "de" | "deu" | "ger" => "German",
        "it" | "ita" => "Italian",
        "pt" | "por" => "Portuguese",
        "ru" | "rus" => "Russian",
        "zh" | "zho" | "chi" => "Chinese",
        "ko" | "kor" => "Korean",
        "ar" | "ara" => "Arabic",
        "he" | "heb" => "Hebrew",
        "hi" | "hin" => "Hindi",
        "bn" | "ben" => "Bengali",
        "id" | "ind" => "Indonesian",
        "th" | "tha" => "Thai",
        "vi" | "vie" => "Vietnamese",
        "nl" | "nld" | "dut" => "Dutch",
        "pl" | "pol" => "Polish",
        "tr" | "tur" => "Turkish",
        "uk" | "ukr" => "Ukrainian",
        "sv" | "swe" => "Swedish",
        "no" | "nor" => "Norwegian",
        "da" | "dan" => "Danish",
        "fi" | "fin" => "Finnish",
        "el" | "ell" | "gre" => "Greek",
        "cs" | "ces" | "cze" => "Czech",
        "hu" | "hun" => "Hungarian",
        "ro" | "ron" | "rum" => "Romanian",
        _ => "",
    }
}

pub fn format_audio_track_label(track: &AudioTrack, index: usize) -> String {
    let cleaned_title = track.title.as_ref().and_then(|t| {
        let trimmed = t.trim();
        if trimmed.is_empty() {
            return None;
        }
        let first_seg = if trimmed.contains(" / ") {
            trimmed.split(" / ").next().unwrap_or(trimmed).trim()
        } else {
            trimmed.split('/').next().unwrap_or(trimmed).trim()
        };
        if first_seg.is_empty() {
            Some(trimmed.to_string())
        } else if first_seg.chars().count() > 65 {
            Some(format!(
                "{}...",
                first_seg.chars().take(62).collect::<String>()
            ))
        } else {
            Some(first_seg.to_string())
        }
    });

    let lang_display = track.lang.as_ref().and_then(|l| {
        let display = language_display_name(l);
        if !display.is_empty() {
            Some(display.to_string())
        } else {
            let trimmed = l.trim().to_ascii_uppercase();
            if !trimmed.is_empty() {
                Some(trimmed)
            } else {
                None
            }
        }
    });

    match (cleaned_title, lang_display) {
        (Some(title), Some(lang)) => {
            if title
                .to_ascii_lowercase()
                .contains(&lang.to_ascii_lowercase())
            {
                title
            } else {
                format!("{lang} ({title})")
            }
        }
        (Some(title), None) => title,
        (None, Some(lang)) => lang,
        (None, None) => {
            if track.id > 0 {
                format!("Track {}", track.id)
            } else {
                format!("Track {}", index + 1)
            }
        }
    }
}

pub fn format_subtitle_track_label(track: &SubtitleTrack, index: usize) -> String {
    let lang_display = track.lang.as_ref().and_then(|l| {
        let display = language_display_name(l);
        if !display.is_empty() {
            Some(display.to_string())
        } else if l.eq_ignore_ascii_case("enm") {
            Some("English".to_string())
        } else {
            let trimmed = l.trim().to_ascii_uppercase();
            if !trimmed.is_empty() {
                Some(trimmed)
            } else {
                None
            }
        }
    });

    let cleaned_title = track.title.as_ref().and_then(|t| {
        let trimmed = t.trim();
        if trimmed.is_empty() {
            return None;
        }
        let lower = trimmed.to_ascii_lowercase();
        let qualifier = if lower.contains("honorific") || lower.contains("honorofic") {
            Some("Honorifics")
        } else if lower.contains("sign") || lower.contains("song") {
            Some("Signs & Songs")
        } else if lower.contains("sdh") {
            Some("SDH")
        } else if lower.contains("forced") {
            Some("Forced")
        } else {
            None
        };

        let first_seg = if trimmed.contains(" / ") {
            trimmed.split(" / ").next().unwrap_or(trimmed).trim()
        } else {
            trimmed.split('/').next().unwrap_or(trimmed).trim()
        };
        let first_lower = first_seg.to_ascii_lowercase();
        if let Some(q) = qualifier {
            let already_has_qualifier = match q {
                "Honorifics" => {
                    first_lower.contains("honorific") || first_lower.contains("honorofic")
                }
                "Signs & Songs" => first_lower.contains("sign") || first_lower.contains("song"),
                "SDH" => first_lower.contains("sdh"),
                "Forced" => first_lower.contains("forced"),
                _ => first_lower.contains(&q.to_ascii_lowercase()),
            };
            if already_has_qualifier {
                Some(first_seg.to_string())
            } else {
                Some(format!("{first_seg} - {q}"))
            }
        } else if first_seg.is_empty() {
            Some(trimmed.to_string())
        } else if first_seg.chars().count() > 65 {
            Some(format!(
                "{}...",
                first_seg.chars().take(62).collect::<String>()
            ))
        } else {
            Some(first_seg.to_string())
        }
    });

    match (cleaned_title, lang_display) {
        (Some(title), Some(lang)) => {
            if title
                .to_ascii_lowercase()
                .contains(&lang.to_ascii_lowercase())
            {
                title
            } else {
                format!("{lang} ({title})")
            }
        }
        (Some(title), None) => title,
        (None, Some(lang)) => lang,
        (None, None) => {
            if track.id > 0 {
                format!("Track {}", track.id)
            } else {
                format!("Track {}", index + 1)
            }
        }
    }
}

pub fn language_aliases(name_or_code: &str) -> Vec<String> {
    let lower = name_or_code.trim().to_ascii_lowercase();
    let mut aliases = Vec::new();
    match lower.as_str() {
        "ja" | "jpn" | "jp" | "japanese" => {
            aliases.extend(vec![
                "ja".into(),
                "jpn".into(),
                "jp".into(),
                "japanese".into(),
            ]);
        }
        "en" | "eng" | "english" => {
            aliases.extend(vec!["en".into(), "eng".into(), "english".into()]);
        }
        "es" | "spa" | "spanish" => {
            aliases.extend(vec!["es".into(), "spa".into(), "spanish".into()]);
        }
        "fr" | "fra" | "fre" | "french" => {
            aliases.extend(vec![
                "fr".into(),
                "fra".into(),
                "fre".into(),
                "french".into(),
            ]);
        }
        "de" | "deu" | "ger" | "german" => {
            aliases.extend(vec![
                "de".into(),
                "deu".into(),
                "ger".into(),
                "german".into(),
            ]);
        }
        "it" | "ita" | "italian" => {
            aliases.extend(vec!["it".into(), "ita".into(), "italian".into()]);
        }
        "pt" | "por" | "portuguese" => {
            aliases.extend(vec!["pt".into(), "por".into(), "portuguese".into()]);
        }
        "ru" | "rus" | "russian" => {
            aliases.extend(vec!["ru".into(), "rus".into(), "russian".into()]);
        }
        "zh" | "zho" | "chi" | "chinese" => {
            aliases.extend(vec![
                "zh".into(),
                "zho".into(),
                "chi".into(),
                "chinese".into(),
            ]);
        }
        "ko" | "kor" | "korean" => {
            aliases.extend(vec!["ko".into(), "kor".into(), "korean".into()]);
        }
        _ => {
            let display = language_display_name(&lower);
            if !display.is_empty() {
                aliases.push(display.to_ascii_lowercase());
            }
            if !lower.is_empty() {
                aliases.push(lower);
            }
        }
    }
    aliases
}

pub fn build_alang_string(preferred: &str) -> String {
    let aliases = language_aliases(preferred);
    aliases.join(",")
}

pub fn get_track_preference_string(track: &AudioTrack) -> String {
    if let Some(ref l) = track.lang {
        let display = language_display_name(l);
        if !display.is_empty() {
            return display.to_string();
        }
        let trimmed = l.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Some(ref _t) = track.title {
        let cleaned = format_audio_track_label(track, 0);
        if !cleaned.is_empty() {
            return cleaned;
        }
    }
    format!("Track {}", track.id)
}

pub fn matches_alias_token(text: &str, alias: &str) -> bool {
    let lower_alias = alias.trim().to_ascii_lowercase();
    if lower_alias.is_empty() {
        return false;
    }
    let lower_text = text.to_ascii_lowercase();
    if lower_alias.len() <= 2 {
        lower_text
            .split(|c: char| !c.is_alphanumeric())
            .any(|w| w == lower_alias)
    } else {
        lower_text.contains(&lower_alias)
    }
}

pub fn find_matching_audio_track(tracks: &[AudioTrack], preferred: &str) -> Option<i64> {
    if preferred.trim().is_empty() {
        return None;
    }
    let aliases = language_aliases(preferred);

    // Pass 1: exact match on track.lang against any alias
    for t in tracks {
        if let Some(ref l) = t.lang {
            let lower_lang = l.trim().to_ascii_lowercase();
            if aliases.iter().any(|a| a == &lower_lang) {
                return Some(t.id);
            }
            let display = language_display_name(&lower_lang).to_ascii_lowercase();
            if !display.is_empty() && aliases.iter().any(|a| a == &display) {
                return Some(t.id);
            }
        }
    }

    // Pass 2: match on track.title containing alias token
    for t in tracks {
        if let Some(ref title) = t.title {
            if aliases.iter().any(|a| matches_alias_token(title, a)) {
                return Some(t.id);
            }
        }
    }

    // Pass 3: match formatted label
    for (i, t) in tracks.iter().enumerate() {
        let label = format_audio_track_label(t, i);
        if aliases.iter().any(|a| matches_alias_token(&label, a)) {
            return Some(t.id);
        }
    }

    None
}

pub fn track_matches_preference(track: &AudioTrack, preferred: &str) -> bool {
    if preferred.trim().is_empty() {
        return false;
    }
    let aliases = language_aliases(preferred);
    if let Some(ref l) = track.lang {
        let lower_lang = l.trim().to_ascii_lowercase();
        if aliases.iter().any(|a| a == &lower_lang) {
            return true;
        }
        let display = language_display_name(&lower_lang).to_ascii_lowercase();
        if !display.is_empty() && aliases.iter().any(|a| a == &display) {
            return true;
        }
    }
    if let Some(ref title) = track.title {
        if aliases.iter().any(|a| matches_alias_token(title, a)) {
            return true;
        }
    }
    let label = format_audio_track_label(track, 0);
    if aliases.iter().any(|a| matches_alias_token(&label, a)) {
        return true;
    }
    false
}
