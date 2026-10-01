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
    pub is_default: bool,
    pub is_forced: bool,
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
        "fa" | "fas" | "per" => "Persian",
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
        "es" | "spa" | "spanish" | "español" | "espanol" => {
            aliases.extend(vec!["es".into(), "spa".into(), "spanish".into()]);
        }
        "fr" | "fra" | "fre" | "french" | "français" | "francais" => {
            aliases.extend(vec![
                "fr".into(),
                "fra".into(),
                "fre".into(),
                "french".into(),
            ]);
        }
        "de" | "deu" | "ger" | "german" | "deutsch" => {
            aliases.extend(vec![
                "de".into(),
                "deu".into(),
                "ger".into(),
                "german".into(),
            ]);
        }
        "it" | "ita" | "italian" | "italiano" => {
            aliases.extend(vec!["it".into(), "ita".into(), "italian".into()]);
        }
        "pt" | "por" | "portuguese" | "português" | "portugues" => {
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
        "nl" | "nld" | "dut" | "dutch" | "nederlands" => {
            aliases.extend(vec![
                "nl".into(),
                "nld".into(),
                "dut".into(),
                "dutch".into(),
            ]);
        }
        "cs" | "ces" | "cze" | "czech" | "čeština" | "cestina" => {
            aliases.extend(vec![
                "cs".into(),
                "ces".into(),
                "cze".into(),
                "czech".into(),
            ]);
        }
        "pl" | "pol" | "polish" | "polski" => {
            aliases.extend(vec!["pl".into(), "pol".into(), "polish".into()]);
        }
        "uk" | "ukr" | "ukrainian" => {
            aliases.extend(vec!["uk".into(), "ukr".into(), "ukrainian".into()]);
        }
        "sv" | "swe" | "swedish" | "svenska" => {
            aliases.extend(vec!["sv".into(), "swe".into(), "swedish".into()]);
        }
        "tr" | "tur" | "turkish" | "türkçe" | "turkce" => {
            aliases.extend(vec!["tr".into(), "tur".into(), "turkish".into()]);
        }
        "ar" | "ara" | "arabic" => {
            aliases.extend(vec!["ar".into(), "ara".into(), "arabic".into()]);
        }
        "he" | "heb" | "hebrew" => {
            aliases.extend(vec!["he".into(), "heb".into(), "hebrew".into()]);
        }
        "fa" | "fas" | "per" | "persian" | "farsi" => {
            aliases.extend(vec![
                "fa".into(),
                "fas".into(),
                "per".into(),
                "persian".into(),
            ]);
        }
        "hi" | "hin" | "hindi" => {
            aliases.extend(vec!["hi".into(), "hin".into(), "hindi".into()]);
        }
        "bn" | "ben" | "bengali" => {
            aliases.extend(vec!["bn".into(), "ben".into(), "bengali".into()]);
        }
        "th" | "tha" | "thai" => {
            aliases.extend(vec!["th".into(), "tha".into(), "thai".into()]);
        }
        "vi" | "vie" | "vietnamese" => {
            aliases.extend(vec!["vi".into(), "vie".into(), "vietnamese".into()]);
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

pub fn build_slang_string_with_fallback(
    preferred: Option<&str>,
    i18n_lang: Option<&str>,
) -> String {
    let mut all_aliases = Vec::new();

    let mut add_lang = |lang: &str| {
        for a in language_aliases(lang) {
            if !all_aliases.contains(&a) {
                all_aliases.push(a);
            }
        }
    };

    if let Some(pref) = preferred.filter(|s| !s.trim().is_empty()) {
        add_lang(pref);
    }
    if let Some(i18n) = i18n_lang.filter(|s| !s.trim().is_empty()) {
        add_lang(i18n);
    }
    add_lang("English");

    all_aliases.join(",")
}

pub fn build_slang_string(preferred: &str) -> String {
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

pub fn get_subtitle_track_preference_string(track: &SubtitleTrack) -> String {
    if let Some(ref l) = track.lang {
        let display = language_display_name(l);
        if !display.is_empty() {
            return display.to_string();
        }
        if l.eq_ignore_ascii_case("enm") {
            return "English".to_string();
        }
        let trimmed = l.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Some(ref _t) = track.title {
        let cleaned = format_subtitle_track_label(track, 0);
        if !cleaned.is_empty() {
            return cleaned;
        }
    }
    format!("Track {}", track.id)
}

pub fn subtitle_track_matches_preference(track: &SubtitleTrack, preferred: &str) -> bool {
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
        if lower_lang == "enm"
            && aliases
                .iter()
                .any(|a| a == "en" || a == "eng" || a == "english")
        {
            return true;
        }
    }
    if let Some(ref title) = track.title {
        if aliases.iter().any(|a| matches_alias_token(title, a)) {
            return true;
        }
    }
    let label = format_subtitle_track_label(track, 0);
    if aliases.iter().any(|a| matches_alias_token(&label, a)) {
        return true;
    }
    false
}

pub fn is_signs_or_songs_track(track: &SubtitleTrack) -> bool {
    track.title.as_ref().map_or(false, |title| {
        let l = title.to_ascii_lowercase();
        l.contains("sign") || l.contains("song")
    })
}

pub fn is_forced_track(track: &SubtitleTrack) -> bool {
    if track.is_forced {
        return true;
    }
    track.title.as_ref().map_or(false, |title| {
        let l = title.to_ascii_lowercase();
        l.contains("forced") || l.contains("forzado") || l.contains("forçat")
    })
}

pub fn find_matching_subtitle_track(tracks: &[SubtitleTrack], preferred: &str) -> Option<i64> {
    if preferred.trim().is_empty() || tracks.is_empty() {
        return None;
    }

    let matching_tracks: Vec<&SubtitleTrack> = tracks
        .iter()
        .filter(|t| subtitle_track_matches_preference(t, preferred))
        .collect();

    if matching_tracks.is_empty() {
        return None;
    }

    // Score matching tracks to pick the best one:
    // Prefer external files, full dialogue, container default, non-signs, non-forced.
    let best = matching_tracks.iter().max_by_key(|t| {
        let mut score = 100i32;
        if !is_signs_or_songs_track(t) {
            score += 50;
        }
        if !is_forced_track(t) {
            score += 20;
        }
        if t.is_default {
            score += 15;
        }
        if t.external_filename.is_some() {
            score += 25;
        }
        if let Some(ref title) = t.title {
            let l = title.to_ascii_lowercase();
            if l.contains("full") {
                score += 10;
            }
        }
        score
    });

    best.map(|t| t.id)
}

pub fn select_best_subtitle_track_with_fallback(
    tracks: &[SubtitleTrack],
    preferred: Option<&str>,
    i18n_lang: Option<&str>,
) -> Option<i64> {
    if tracks.is_empty() {
        return None;
    }

    // 1. Try preferred subtitle language if known
    if let Some(pref) = preferred.filter(|s| !s.trim().is_empty()) {
        if let Some(id) = find_matching_subtitle_track(tracks, pref) {
            return Some(id);
        }
    }

    // 2. If preferred is not known or not matched, use the "language" i18n setting
    let effective_i18n = i18n_lang.filter(|s| !s.trim().is_empty()).or_else(|| {
        if preferred.is_none() {
            Some("en")
        } else {
            None
        }
    });

    if let Some(i18n) = effective_i18n {
        let is_same_as_pref = preferred.map_or(false, |p| p.eq_ignore_ascii_case(i18n));
        if !is_same_as_pref {
            if let Some(id) = find_matching_subtitle_track(tracks, i18n) {
                return Some(id);
            }
        }
    }

    // 3. Fallback to English if neither preferred nor i18n matched English
    let already_tried_english = preferred.map_or(false, |p| {
        p.eq_ignore_ascii_case("English")
            || p.eq_ignore_ascii_case("eng")
            || p.eq_ignore_ascii_case("en")
    }) || effective_i18n.map_or(false, |i| {
        i.eq_ignore_ascii_case("English")
            || i.eq_ignore_ascii_case("eng")
            || i.eq_ignore_ascii_case("en")
    });

    if !already_tried_english {
        if let Some(id) = find_matching_subtitle_track(tracks, "English") {
            return Some(id);
        }
    }

    // 4. Try external subtitle file
    if let Some(ext) = tracks.iter().find(|t| t.external_filename.is_some()) {
        return Some(ext.id);
    }

    // 5. Try container default track (preferring full dialogue over signs/songs)
    if let Some(def) = tracks
        .iter()
        .find(|t| t.is_default && !is_signs_or_songs_track(t))
    {
        return Some(def.id);
    }
    if let Some(def) = tracks.iter().find(|t| t.is_default) {
        return Some(def.id);
    }

    // 6. Try track with "full" in title
    if let Some(full) = tracks.iter().find(|t| {
        t.title.as_ref().map_or(false, |title| {
            let l = title.to_ascii_lowercase();
            l.contains("full") && !is_signs_or_songs_track(t)
        })
    }) {
        return Some(full.id);
    }

    // 7. First track that is not signs/songs
    if let Some(normal) = tracks.iter().find(|t| !is_signs_or_songs_track(t)) {
        return Some(normal.id);
    }

    // 8. First track
    tracks.first().map(|t| t.id)
}

pub fn select_best_subtitle_track(
    tracks: &[SubtitleTrack],
    preferred: Option<&str>,
) -> Option<i64> {
    select_best_subtitle_track_with_fallback(tracks, preferred, None)
}
