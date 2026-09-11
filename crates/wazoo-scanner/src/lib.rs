use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use regex::Regex;
use serde::Deserialize;
use tokio::sync::mpsc;
use wazoo_core::{Database, VideoRecord};

pub const VIDEO_EXTENSIONS: &[&str] = &["mkv", "mp4", "avi", "mov", "webm"];

pub fn is_video_file<P: AsRef<Path>>(path: P) -> bool {
    let ext = path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext {
        Some(e) => VIDEO_EXTENSIONS.contains(&e.as_str()),
        None => false,
    }
}

pub fn clean_video_name(filename: &str) -> String {
    let bracket_re = Regex::new(r"\[.*?\]").unwrap();
    let spaces_re = Regex::new(r"\s{2,}").unwrap();

    let without_brackets = bracket_re.replace_all(filename, " ");
    let cleaned = without_brackets.replace(['.', '_'], " ");
    let single_spaced = spaces_re.replace_all(&cleaned, " ");
    let trimmed = single_spaced.trim().trim_matches('-');

    if trimmed.is_empty() {
        filename.to_string()
    } else {
        trimmed.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct VideoMetadata {
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub has_subtitles: bool,
}

#[derive(Deserialize)]
struct FfprobeOutput {
    streams: Option<Vec<FfprobeStream>>,
    format: Option<FfprobeFormat>,
}

#[derive(Deserialize)]
struct FfprobeStream {
    codec_name: Option<String>,
    codec_type: Option<String>,
    profile: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    duration: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

pub fn probe_video_metadata<P: AsRef<Path>>(path: P, ffprobe_bin: Option<&str>) -> VideoMetadata {
    let fallback = VideoMetadata {
        codec: "unknown".to_string(),
        width: 0,
        height: 0,
        duration: 0.0,
        has_subtitles: false,
    };

    let bin = ffprobe_bin.unwrap_or("ffprobe");
    let output = Command::new(bin)
        .args([
            "-v", "error",
            "-analyzeduration", "100000",
            "-probesize", "5000000",
            "-show_entries", "stream=codec_name,profile,width,height,codec_type:format=duration",
            "-of", "json",
            path.as_ref().to_str().unwrap_or_default(),
        ])
        .output();

    let output = match output {
        Ok(out) if out.status.success() => out.stdout,
        _ => return fallback,
    };

    let data: FfprobeOutput = match serde_json::from_slice(&output) {
        Ok(d) => d,
        Err(_) => return fallback,
    };

    let mut codec = "unknown".to_string();
    let mut width = 0;
    let mut height = 0;
    let mut stream_duration = 0.0;
    let mut has_subtitles = false;

    if let Some(streams) = data.streams {
        for s in streams {
            let codec_type = s.codec_type.as_deref().unwrap_or("");
            if codec_type == "video" && width == 0 {
                width = s.width.unwrap_or(0);
                height = s.height.unwrap_or(0);
                let codec_raw = s.codec_name.as_deref().unwrap_or("").to_lowercase();
                let profile_raw = s.profile.as_deref().unwrap_or("").to_lowercase();

                if codec_raw.contains("hevc") && profile_raw.contains("main 10") {
                    codec = "hevc-10bit".to_string();
                } else if codec_raw.contains("hevc") {
                    codec = "hevc-8bit".to_string();
                } else if codec_raw.contains("h264") {
                    codec = "h264".to_string();
                } else if codec_raw.contains("av1") {
                    codec = "av1".to_string();
                } else if codec_raw.contains("vp9") {
                    codec = "vp9".to_string();
                } else {
                    codec = codec_raw;
                }

                if let Some(dur_str) = s.duration {
                    stream_duration = dur_str.parse().unwrap_or(0.0);
                }
            } else if codec_type == "subtitle" {
                has_subtitles = true;
            }
        }
    }

    let format_duration = data
        .format
        .and_then(|f| f.duration)
        .and_then(|d| d.parse::<f64>().ok())
        .unwrap_or(0.0);

    let duration = if format_duration > 0.0 {
        format_duration
    } else {
        stream_duration
    };

    VideoMetadata {
        codec,
        width,
        height,
        duration,
        has_subtitles,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanStage {
    Listing,
    Indexing,
}

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub stage: ScanStage,
    pub processed: usize,
    pub total: usize,
    pub percent: usize,
    pub current_name: String,
}

pub struct Scanner {
    #[allow(dead_code)]
    ffprobe_bin: Option<String>,
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Scanner {
    pub fn new(ffprobe_bin: Option<String>) -> Self {
        Self { ffprobe_bin }
    }

    pub fn discover_files(folders: &[String]) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for folder in folders {
            let path = Path::new(folder);
            if !path.is_dir() {
                continue;
            }

            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() && is_video_file(p) {
                    files.push(p.to_path_buf());
                }
            }
        }
        files
    }

    pub async fn scan_and_index(
        &self,
        folders: &[String],
        db_path: PathBuf,
        progress_tx: Option<mpsc::Sender<ScanProgress>>,
    ) -> Result<usize, String> {
        let mut files = Vec::new();
        for folder in folders {
            let path = Path::new(folder);
            if !path.is_dir() {
                continue;
            }

            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() && is_video_file(p) {
                    files.push(p.to_path_buf());
                    if let Some(ref tx) = progress_tx {
                        if files.len() % 25 == 0 {
                            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                            let _ = tx.send(ScanProgress {
                                stage: ScanStage::Listing,
                                processed: files.len(),
                                total: 0,
                                percent: 0,
                                current_name: name,
                            }).await;
                        }
                    }
                }
            }
        }

        let total = files.len();
        if total == 0 {
            if let Some(ref tx) = progress_tx {
                let _ = tx.send(ScanProgress {
                    stage: ScanStage::Listing,
                    processed: 0,
                    total: 0,
                    percent: 100,
                    current_name: String::new(),
                }).await;
            }
            return Ok(0);
        }

        if let Some(ref tx) = progress_tx {
            let _ = tx.send(ScanProgress {
                stage: ScanStage::Listing,
                processed: total,
                total,
                percent: 100,
                current_name: format!("{total} files discovered"),
            }).await;
        }

        let mut db = Database::open(&db_path).map_err(|e| e.to_string())?;
        let mut records = Vec::with_capacity(files.len());
        let mut existing_paths = Vec::with_capacity(files.len());

        for (idx, file) in files.into_iter().enumerate() {
            let path_str = file.to_string_lossy().to_string();
            let file_stem = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let cleaned_name = clean_video_name(file_stem);

            existing_paths.push(path_str.clone());

            let record = VideoRecord {
                id: 0,
                name: cleaned_name.clone(),
                path: path_str,
                codec: "native".to_string(),
                width: 0,
                height: 0,
                duration: 0.0,
                has_subtitles: false,
                created_at: chrono::Utc::now().timestamp(),
            };

            records.push(record);

            let processed = idx + 1;
            let percent = (processed * 100) / total;

            if let Some(ref tx) = progress_tx {
                if processed % 15 == 0 || processed == total {
                    let _ = tx.send(ScanProgress {
                        stage: ScanStage::Indexing,
                        processed,
                        total,
                        percent,
                        current_name: cleaned_name,
                    }).await;
                }
            }
        }

        let inserted = db.batch_insert_videos(&records).map_err(|e| e.to_string())?;
        let _ = db.prune_missing_videos(&existing_paths);
        Ok(inserted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_video_name() {
        assert_eq!(clean_video_name("[1080p] My_Great.Movie-Part1"), "My Great Movie-Part1");
        assert_eq!(clean_video_name("[Group] Nature_Documentary_[HEVC]"), "Nature Documentary");
    }

    #[test]
    fn test_is_video_file() {
        assert!(is_video_file("test.mp4"));
        assert!(is_video_file("test.mkv"));
        assert!(is_video_file("test.MKV"));
        assert!(!is_video_file("test.txt"));
        assert!(!is_video_file("test.png"));
    }

    #[tokio::test]
    async fn test_scanner_no_ffprobe() {
        let tmp = std::env::temp_dir().join(format!("wazoo_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&tmp);
        let test_file = tmp.join("Test.Video.2026.mkv");
        let _ = std::fs::write(&test_file, b"dummy video content");
        let db_file = tmp.join("test.db");

        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let scanner = Scanner::default();
        let folders = vec![tmp.to_string_lossy().to_string()];
        let res = scanner.scan_and_index(&folders, db_file, Some(tx)).await;
        assert_eq!(res.unwrap(), 1);

        let mut progress_count = 0;
        while let Ok(progress) = rx.try_recv() {
            progress_count += 1;
            assert!(progress.total <= 1);
        }
        assert!(progress_count >= 1);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
