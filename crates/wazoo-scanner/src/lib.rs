/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Media Scanner & Library Indexer
 *
 * Discovers video files across media directories, cleans filenames, extracts metadata,
 * and batches database updates with real-time progress reporting.
 */

use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use tokio::sync::mpsc;
use walkdir::WalkDir;
use wazoo_core::{Database, VideoRecord};

pub const VIDEO_EXTENSIONS: &[&str] = &["mkv", "mp4", "avi", "mov", "webm"];

static RE_BRACKETS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[.*?\]").unwrap());
static RE_SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s{2,}").unwrap());

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
    let without_brackets = RE_BRACKETS.replace_all(filename, " ");
    let cleaned = without_brackets.replace(['.', '_'], " ");
    let single_spaced = RE_SPACES.replace_all(&cleaned, " ");
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
            "-v",
            "error",
            "-analyzeduration",
            "100000",
            "-probesize",
            "5000000",
            "-show_entries",
            "stream=codec_name,profile,width,height,codec_type:format=duration",
            "-of",
            "json",
            "--",
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
    pub files_found: usize,
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
        self.scan_and_index_with_cancel(folders, db_path, progress_tx, None)
            .await
    }

    #[allow(clippy::manual_checked_ops)]
    pub async fn scan_and_index_with_cancel(
        &self,
        folders: &[String],
        db_path: PathBuf,
        progress_tx: Option<mpsc::Sender<ScanProgress>>,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Result<usize, String> {
        let is_cancelled = || {
            cancel
                .as_ref()
                .map(|c| c.load(Ordering::Relaxed))
                .unwrap_or(false)
        };

        if is_cancelled() {
            return Err("Scan cancelled".to_string());
        }

        let mut files = Vec::new();
        let mut subdirs = Vec::new();
        let mut seen_files = std::collections::HashSet::new();

        // 1. Initial pass: Read immediate entries of each configured root folder
        for folder in folders {
            if is_cancelled() {
                return Err("Scan cancelled".to_string());
            }
            let path = Path::new(folder);
            if !path.is_dir() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if p.is_dir() {
                        subdirs.push(p);
                    } else if is_video_file(&p) && seen_files.insert(p.clone()) {
                        files.push(p);
                    }
                }
            }
        }

        // Expand container directories if we have a small set of directories (< 25)
        // so progress is tracked per show/movie folder rather than across an entire disk or root.
        while subdirs.len() < 25 {
            if is_cancelled() {
                return Err("Scan cancelled".to_string());
            }
            let mut any_expanded = false;
            let mut next_subdirs = Vec::new();

            for dir in subdirs {
                if is_cancelled() {
                    return Err("Scan cancelled".to_string());
                }
                let mut child_dirs = Vec::new();
                let mut child_videos = Vec::new();

                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let p = entry.path();
                        if p.is_dir() {
                            child_dirs.push(p);
                        } else if is_video_file(&p) {
                            child_videos.push(p);
                        }
                    }
                }

                if !child_dirs.is_empty() && (child_videos.is_empty() || child_dirs.len() > 1) {
                    for v in child_videos {
                        if seen_files.insert(v.clone()) {
                            files.push(v);
                        }
                    }
                    next_subdirs.extend(child_dirs);
                    any_expanded = true;
                } else {
                    next_subdirs.push(dir);
                }
            }

            subdirs = next_subdirs;
            if !any_expanded {
                break;
            }
        }

        let total_dirs = subdirs.len();
        if total_dirs > 0 {
            for (idx, subdir) in subdirs.into_iter().enumerate() {
                if is_cancelled() {
                    return Err("Scan cancelled".to_string());
                }
                let dir_name = subdir
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                let percent = (idx * 100) / total_dirs;

                if let Some(ref tx) = progress_tx {
                    let _ = tx
                        .send(ScanProgress {
                            stage: ScanStage::Listing,
                            processed: idx,
                            total: total_dirs,
                            percent,
                            current_name: dir_name.clone(),
                            files_found: files.len(),
                        })
                        .await;
                }

                let prev_files = files.len();
                for entry in WalkDir::new(&subdir).into_iter().filter_map(|e| e.ok()) {
                    if is_cancelled() {
                        return Err("Scan cancelled".to_string());
                    }
                    let p = entry.path();
                    if p.is_file() && is_video_file(p) {
                        let pb = p.to_path_buf();
                        if seen_files.insert(pb.clone()) {
                            files.push(pb);
                            if (files.len() - prev_files) % 15 == 0 {
                                if let Some(ref tx) = progress_tx {
                                    let _ = tx
                                        .send(ScanProgress {
                                            stage: ScanStage::Listing,
                                            processed: idx,
                                            total: total_dirs,
                                            percent,
                                            current_name: dir_name.clone(),
                                            files_found: files.len(),
                                        })
                                        .await;
                                }
                            }
                        }
                    }
                }

                let end_percent = ((idx + 1) * 100) / total_dirs;
                if let Some(ref tx) = progress_tx {
                    let _ = tx
                        .send(ScanProgress {
                            stage: ScanStage::Listing,
                            processed: idx + 1,
                            total: total_dirs,
                            percent: end_percent,
                            current_name: dir_name,
                            files_found: files.len(),
                        })
                        .await;
                }
            }
        } else if let Some(ref tx) = progress_tx {
            let _ = tx
                .send(ScanProgress {
                    stage: ScanStage::Listing,
                    processed: 1,
                    total: 1,
                    percent: 100,
                    current_name: format!("{} files discovered", files.len()),
                    files_found: files.len(),
                })
                .await;
        }

        if is_cancelled() {
            return Err("Scan cancelled".to_string());
        }

        let total = files.len();
        if total == 0 {
            return Ok(0);
        }

        let mut db = Database::open(&db_path).map_err(|e| e.to_string())?;
        let mut records = Vec::with_capacity(files.len());
        let mut existing_paths = Vec::with_capacity(files.len());

        for (idx, file) in files.into_iter().enumerate() {
            if is_cancelled() {
                return Err("Scan cancelled".to_string());
            }
            let path_str = file.to_string_lossy().to_string();
            let file_stem = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let cleaned_name = clean_video_name(file_stem);

            existing_paths.push(path_str.clone());

            let record = VideoRecord::new(0, cleaned_name.clone(), path_str);

            records.push(record);

            let processed = idx + 1;
            let percent = (processed * 100) / total;

            if let Some(ref tx) = progress_tx {
                if processed % 15 == 0 || processed == total {
                    let _ = tx
                        .send(ScanProgress {
                            stage: ScanStage::Indexing,
                            processed,
                            total,
                            percent,
                            current_name: cleaned_name,
                            files_found: total,
                        })
                        .await;
                }
            }
        }

        if is_cancelled() {
            return Err("Scan cancelled".to_string());
        }

        let inserted = db
            .batch_insert_videos(&records)
            .map_err(|e| e.to_string())?;
        let _ = db.prune_missing_videos(&existing_paths);
        Ok(inserted)
    }
}

