/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Subtitle Transcript Engine & Parser
 * 
 * Extracts and parses subtitle cues from external sidecar files (.srt, .vtt, .ass, .ssa)
 * or embedded container streams via ffmpeg, providing timestamped, cleaned text cues
 * for interactive seeking and real-time transcript synchronization.
 */

use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubtitleCue {
    pub start_secs: f64,
    pub end_secs: f64,
    pub text: String,
}

/// Parses standard subtitle timestamps (00:01:23,450 or 00:01:23.450 or 0:01:23.45) into seconds.
pub fn parse_timestamp(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let parts: Vec<&str> = s.split(':').collect();
    let (hours, mins, sec_part) = match parts.len() {
        3 => (parts[0].parse::<f64>().ok()?, parts[1].parse::<f64>().ok()?, parts[2]),
        2 => (0.0, parts[0].parse::<f64>().ok()?, parts[1]),
        1 => (0.0, 0.0, parts[0]),
        _ => return None,
    };

    let sec_clean = sec_part.replace(',', ".");
    let secs = sec_clean.parse::<f64>().ok()?;

    if !hours.is_finite() || !mins.is_finite() || !secs.is_finite() || hours < 0.0 || mins < 0.0 || secs < 0.0 {
        return None;
    }

    let total = hours * 3600.0 + mins * 60.0 + secs;
    if !total.is_finite() || total < 0.0 {
        return None;
    }

    Some(total)
}

/// Strips HTML tags, ASS override tags (e.g. {\an8}), and formats line breaks into clean text.
pub fn clean_subtitle_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    let mut in_brace = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            '{' => in_brace = true,
            '}' => in_brace = false,
            '\\' if in_tag || in_brace => {}
            '\\' => {
                if let Some(&next) = chars.peek() {
                    if next == 'N' || next == 'n' || next == 'h' {
                        chars.next();
                        out.push(' ');
                        continue;
                    }
                }
                out.push(c);
            }
            _ if !in_tag && !in_brace => out.push(c),
            _ => {}
        }
    }

    out.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Parses SubRip (.srt) and WebVTT (.vtt) text content into structured cues.
pub fn parse_srt_or_vtt(content: &str) -> Vec<SubtitleCue> {
    let mut cues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0;

    while idx < lines.len() {
        let line = lines[idx].trim();
        if line.contains("-->") {
            let parts: Vec<&str> = line.split("-->").collect();
            if parts.len() == 2 {
                let start_str = parts[0].trim();
                let end_part = parts[1].trim();
                let end_str = end_part.split_whitespace().next().unwrap_or(end_part);

                if let (Some(start), Some(end)) = (parse_timestamp(start_str), parse_timestamp(end_str)) {
                    idx += 1;
                    let mut text_lines = Vec::new();
                    while idx < lines.len() && !lines[idx].trim().is_empty() {
                        text_lines.push(lines[idx].trim());
                        idx += 1;
                    }
                    let raw_text = text_lines.join(" ");
                    let cleaned = clean_subtitle_text(&raw_text);
                    if !cleaned.is_empty() {
                        cues.push(SubtitleCue {
                            start_secs: start,
                            end_secs: end,
                            text: cleaned,
                        });
                    }
                }
            }
        }
        idx += 1;
    }

    cues
}

/// Parses Advanced SubStation Alpha (.ass / .ssa) dialogue lines into structured cues.
pub fn parse_ass(content: &str) -> Vec<SubtitleCue> {
    let mut cues = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Dialogue:") {
            let parts: Vec<&str> = rest.splitn(10, ',').collect();
            if parts.len() == 10 {
                let start_str = parts[1].trim();
                let end_str = parts[2].trim();
                let text_raw = parts[9].trim();

                if let (Some(start), Some(end)) = (parse_timestamp(start_str), parse_timestamp(end_str)) {
                    let cleaned = clean_subtitle_text(text_raw);
                    if !cleaned.is_empty() {
                        cues.push(SubtitleCue {
                            start_secs: start,
                            end_secs: end,
                            text: cleaned,
                        });
                    }
                }
            }
        }
    }

    cues
}

/// Dispatches content to the appropriate subtitle parser based on format headers.
pub fn parse_subtitles(content: &str) -> Vec<SubtitleCue> {
    if content.contains("[Events]") || content.contains("Dialogue:") {
        let ass_cues = parse_ass(content);
        if !ass_cues.is_empty() {
            return ass_cues;
        }
    }
    parse_srt_or_vtt(content)
}

/// Synchronously loads subtitles for a given video path and subtitle stream index.
/// If track_index == 0, first searches for sidecar subtitle files (.srt, .vtt, .ass, etc.) alongside the video.
/// If not found (or if track_index > 0), attempts to demux the subtitle stream using ffmpeg -map 0:s:{track_index}.
pub fn load_subtitles_for_track_sync(video_path: &str, track_index: usize) -> Vec<SubtitleCue> {
    // 1. Attempt to extract embedded subtitle track using ffmpeg -map 0:s:{track_index}
    let map_arg = format!("0:s:{track_index}");
    if let Ok(output) = std::process::Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-protocol_whitelist", "file,crypto",
            "-v", "error",
            "-i", video_path,
            "-map", &map_arg,
            "-f", "srt",
            "-",
        ])
        .output()
    {
        if output.status.success() && !output.stdout.is_empty() {
            let content = String::from_utf8_lossy(&output.stdout);
            let cues = parse_srt_or_vtt(&content);
            if !cues.is_empty() {
                log::info!("Extracted {} subtitle cues via ffmpeg (-map {}) from {}", cues.len(), map_arg, video_path);
                return cues;
            }
        }
    }

    // 2. Fallback for primary track (track 0): check sidecar files (.srt, .vtt, .ass, etc.)
    if track_index == 0 {
        let path = Path::new(video_path);
        if let (Some(parent), Some(stem)) = (path.parent(), path.file_stem()) {
            let stem_str = stem.to_string_lossy();
            let candidates = [
                parent.join(format!("{stem_str}.srt")),
                parent.join(format!("{stem_str}.vtt")),
                parent.join(format!("{stem_str}.ass")),
                parent.join(format!("{stem_str}.ssa")),
            ];

            for cand in candidates {
                if cand.exists() {
                    if let Ok(content) = std::fs::read_to_string(&cand) {
                        let cues = parse_subtitles(&content);
                        if !cues.is_empty() {
                            log::info!("Loaded {} subtitle cues from sidecar {:?}", cues.len(), cand);
                            return cues;
                        }
                    }
                }
            }
        }
    }

    Vec::new()
}

/// Asynchronously loads and parses subtitles for a specific subtitle stream on a background thread.
pub async fn load_subtitles_for_track(video_path: String, track_index: usize) -> Vec<SubtitleCue> {
    tokio::task::spawn_blocking(move || load_subtitles_for_track_sync(&video_path, track_index))
        .await
        .unwrap_or_default()
}

/// Synchronously loads subtitles for the primary track (track 0).
pub fn load_subtitles_sync(video_path: &str) -> Vec<SubtitleCue> {
    load_subtitles_for_track_sync(video_path, 0)
}

/// Asynchronously loads and parses subtitles for the primary track (track 0) on a background thread.
pub async fn load_subtitles(video_path: String) -> Vec<SubtitleCue> {
    load_subtitles_for_track(video_path, 0).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp() {
        assert_eq!(parse_timestamp("00:00:14,760"), Some(14.76));
        assert_eq!(parse_timestamp("01:23.450"), Some(83.45));
        assert_eq!(parse_timestamp("1:02:03.50"), Some(3723.5));
        assert_eq!(parse_timestamp("invalid"), None);
        assert_eq!(parse_timestamp("NaN:NaN:NaN"), None);
        assert_eq!(parse_timestamp("inf"), None);
        assert_eq!(parse_timestamp("-01:00"), None);
    }

    #[test]
    fn test_clean_subtitle_text() {
        let input = r#"<font face="Chalk" size="30">{\an8}Wow, it's windy.\NReally windy!</font>"#;
        assert_eq!(clean_subtitle_text(input), "Wow, it's windy. Really windy!");
    }

    #[test]
    fn test_parse_srt() {
        let srt = r#"
1
00:00:10,000 --> 00:00:14,000
First subtitle line.

2
00:00:15,500 --> 00:00:19,000
Second subtitle line with <i>italics</i>.
"#;
        let cues = parse_srt_or_vtt(srt);
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].start_secs, 10.0);
        assert_eq!(cues[0].end_secs, 14.0);
        assert_eq!(cues[0].text, "First subtitle line.");
        assert_eq!(cues[1].start_secs, 15.5);
        assert_eq!(cues[1].end_secs, 19.0);
        assert_eq!(cues[1].text, "Second subtitle line with italics.");
    }

    #[test]
    fn test_parse_ass() {
        let ass = r#"
[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:14.76,0:00:18.49,Default,,0,0,0,,{\an8}Wow, it's windy.
Dialogue: 0,0:00:20.00,0:00:22.50,Default,,0,0,0,,Second line.
"#;
        let cues = parse_ass(ass);
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].start_secs, 14.76);
        assert_eq!(cues[0].end_secs, 18.49);
        assert_eq!(cues[0].text, "Wow, it's windy.");
        assert_eq!(cues[1].start_secs, 20.0);
        assert_eq!(cues[1].end_secs, 22.5);
        assert_eq!(cues[1].text, "Second line.");
    }
}
