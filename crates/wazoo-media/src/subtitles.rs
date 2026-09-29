/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Subtitle Transcript Engine & Parser
 *
 * Extracts and parses subtitle cues from external sidecar files (.srt, .vtt, .ass, .ssa)
 * or embedded container streams via ffmpeg, providing timestamped, cleaned text cues
 * for interactive seeking and real-time transcript synchronization.
 */

use serde::{Deserialize, Serialize};
use std::path::Path;

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
        3 => (
            parts[0].parse::<f64>().ok()?,
            parts[1].parse::<f64>().ok()?,
            parts[2],
        ),
        2 => (0.0, parts[0].parse::<f64>().ok()?, parts[1]),
        1 => (0.0, 0.0, parts[0]),
        _ => return None,
    };

    let sec_clean = sec_part.replace(',', ".");
    let secs = sec_clean.parse::<f64>().ok()?;

    if !hours.is_finite()
        || !mins.is_finite()
        || !secs.is_finite()
        || hours < 0.0
        || mins < 0.0
        || secs < 0.0
    {
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

                if let (Some(start), Some(end)) =
                    (parse_timestamp(start_str), parse_timestamp(end_str))
                {
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

                if let (Some(start), Some(end)) =
                    (parse_timestamp(start_str), parse_timestamp(end_str))
                {
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

fn run_ffmpeg_subtitle_extract(video_path: &str, map_arg: &str) -> Option<Vec<SubtitleCue>> {
    let output = std::process::Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-protocol_whitelist",
            "file,crypto",
            "-v",
            "error",
            "-i",
            video_path,
            "-map",
            map_arg,
            "-f",
            "srt",
            "-",
        ])
        .output()
        .ok()?;

    if output.status.success() && !output.stdout.is_empty() {
        let content = String::from_utf8_lossy(&output.stdout);
        let cues = parse_srt_or_vtt(&content);
        if !cues.is_empty() {
            return Some(cues);
        }
    }
    None
}

/// Synchronously loads subtitles for a given video path, container stream index (`ff_index`),
/// or subtitle stream index (`sub_index`).
pub fn load_subtitles_for_stream_sync(
    video_path: &str,
    ff_index: Option<i64>,
    sub_index: usize,
) -> Vec<SubtitleCue> {
    // 1. If ff_index is provided from container metadata, map the exact container stream: -map 0:{ff_index}
    if let Some(idx) = ff_index {
        let map_arg = format!("0:{idx}");
        if let Some(cues) = run_ffmpeg_subtitle_extract(video_path, &map_arg) {
            log::info!(
                "Extracted {} subtitle cues via ffmpeg (-map {}) from {}",
                cues.len(),
                map_arg,
                video_path
            );
            return cues;
        }
    }

    // 2. Otherwise/fallback: map by subtitle stream index: -map 0:s:{sub_index}
    let map_arg = format!("0:s:{sub_index}");
    if let Some(cues) = run_ffmpeg_subtitle_extract(video_path, &map_arg) {
        log::info!(
            "Extracted {} subtitle cues via ffmpeg (-map {}) from {}",
            cues.len(),
            map_arg,
            video_path
        );
        return cues;
    }

    // 3. Fallback for primary track (sub_index == 0): check sidecar files (.srt, .vtt, .ass, etc.)
    if sub_index == 0 {
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
                            log::info!(
                                "Loaded {} subtitle cues from sidecar {:?}",
                                cues.len(),
                                cand
                            );
                            return cues;
                        }
                    }
                }
            }
        }
    }

    Vec::new()
}

/// Asynchronously extracts subtitle cues via ffmpeg using Tokio async process management.
/// Uses kill_on_drop to immediately terminate stalled or superseded ffmpeg extractions,
/// executes with a timeout to avoid hanging, and offloads string parsing onto the blocking pool.
pub async fn run_ffmpeg_subtitle_extract_async(
    video_path: &str,
    map_arg: &str,
) -> Option<Vec<SubtitleCue>> {
    let mut cmd = tokio::process::Command::new("ffmpeg");
    cmd.args([
        "-nostdin",
        "-protocol_whitelist",
        "file,crypto",
        "-v",
        "error",
        "-i",
        video_path,
        "-map",
        map_arg,
        "-f",
        "srt",
        "-",
    ]);
    cmd.kill_on_drop(true);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let output = tokio::time::timeout(std::time::Duration::from_secs(15), cmd.output())
        .await
        .ok()?
        .ok()?;

    if output.status.success() && !output.stdout.is_empty() {
        let content = String::from_utf8_lossy(&output.stdout).to_string();
        let cues = tokio::task::spawn_blocking(move || parse_srt_or_vtt(&content))
            .await
            .unwrap_or_default();
        if !cues.is_empty() {
            return Some(cues);
        }
    }
    None
}

/// Asynchronously loads subtitles for a given track, prioritizing external sidecar files if specified,
/// followed by container stream extraction (`ff_index`), then stream index fallback (`sub_index`).
pub async fn load_subtitles_for_track_details(
    video_path: String,
    external_filename: Option<String>,
    ff_index: Option<i64>,
    sub_index: usize,
) -> Vec<SubtitleCue> {
    // 1. If an external subtitle file is explicitly specified, load and parse it asynchronously off-thread
    if let Some(ext_file) = external_filename {
        if tokio::fs::try_exists(&ext_file).await.unwrap_or(false) {
            if let Ok(content) = tokio::fs::read_to_string(&ext_file).await {
                let cues = tokio::task::spawn_blocking(move || parse_subtitles(&content))
                    .await
                    .unwrap_or_default();
                if !cues.is_empty() {
                    return cues;
                }
            }
        }
    }

    // 2. Otherwise load via stream index / embedded ffmpeg extraction
    load_subtitles_for_stream(video_path, ff_index, sub_index).await
}

/// Asynchronously loads and parses subtitles for a container stream or subtitle stream index.
/// Employs non-blocking Tokio child process execution, async filesystem I/O, and offloads
/// CPU-bound subtitle parsing to Tokio's blocking thread pool.
pub async fn load_subtitles_for_stream(
    video_path: String,
    ff_index: Option<i64>,
    sub_index: usize,
) -> Vec<SubtitleCue> {
    // 1. If ff_index is provided from container metadata, map the exact container stream: -map 0:{ff_index}
    if let Some(idx) = ff_index {
        let map_arg = format!("0:{idx}");
        if let Some(cues) = run_ffmpeg_subtitle_extract_async(&video_path, &map_arg).await {
            log::info!(
                "Extracted {} subtitle cues via ffmpeg (-map {}) from {}",
                cues.len(),
                map_arg,
                video_path
            );
            return cues;
        }
    }

    // 2. Otherwise/fallback: map by subtitle stream index: -map 0:s:{sub_index}
    let map_arg = format!("0:s:{sub_index}");
    if let Some(cues) = run_ffmpeg_subtitle_extract_async(&video_path, &map_arg).await {
        log::info!(
            "Extracted {} subtitle cues via ffmpeg (-map {}) from {}",
            cues.len(),
            map_arg,
            video_path
        );
        return cues;
    }

    // 3. Fallback for primary track (sub_index == 0): check sidecar files (.srt, .vtt, .ass, etc.)
    if sub_index == 0 {
        let path = Path::new(&video_path);
        if let (Some(parent), Some(stem)) = (path.parent(), path.file_stem()) {
            let stem_str = stem.to_string_lossy();
            let candidates = [
                parent.join(format!("{stem_str}.srt")),
                parent.join(format!("{stem_str}.vtt")),
                parent.join(format!("{stem_str}.ass")),
                parent.join(format!("{stem_str}.ssa")),
            ];

            for cand in candidates {
                if tokio::fs::try_exists(&cand).await.unwrap_or(false) {
                    if let Ok(content) = tokio::fs::read_to_string(&cand).await {
                        let cues = tokio::task::spawn_blocking(move || parse_subtitles(&content))
                            .await
                            .unwrap_or_default();
                        if !cues.is_empty() {
                            log::info!(
                                "Loaded {} subtitle cues from sidecar {:?}",
                                cues.len(),
                                cand
                            );
                            return cues;
                        }
                    }
                }
            }
        }
    }

    Vec::new()
}

/// Synchronously loads subtitles for a given video path and subtitle stream index.
pub fn load_subtitles_for_track_sync(video_path: &str, track_index: usize) -> Vec<SubtitleCue> {
    load_subtitles_for_stream_sync(video_path, None, track_index)
}

/// Asynchronously loads and parses subtitles for a specific subtitle stream on a background thread.
pub async fn load_subtitles_for_track(video_path: String, track_index: usize) -> Vec<SubtitleCue> {
    load_subtitles_for_stream(video_path, None, track_index).await
}

/// Synchronously loads subtitles for the primary track (track 0).
pub fn load_subtitles_sync(video_path: &str) -> Vec<SubtitleCue> {
    load_subtitles_for_track_sync(video_path, 0)
}

/// Asynchronously loads and parses subtitles for the primary track (track 0) on a background thread.
pub async fn load_subtitles(video_path: String) -> Vec<SubtitleCue> {
    load_subtitles_for_track(video_path, 0).await
}
