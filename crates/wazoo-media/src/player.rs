use std::time::{Duration, Instant};
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer::prelude::*;
use rand::Rng;
use url::Url;

pub type PlayerId = usize;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: PlayerId,
    pub path: String,
    pub name: String,
    pub duration: Duration,
    pub position: Duration,
    pub volume: f64,
    pub is_muted: bool,
    pub is_playing: bool,
    pub last_checked_pos: Duration,
    pub stuck_count: usize,
}

impl PlayerState {
    pub fn new(id: PlayerId, path: String, name: String) -> Self {
        Self {
            id,
            path,
            name,
            duration: Duration::ZERO,
            position: Duration::ZERO,
            volume: 1.0,
            is_muted: false,
            is_playing: true,
            last_checked_pos: Duration::ZERO,
            stuck_count: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BufferConfig {
    pub duration_secs: u32,
    pub size_mb: u32,
    pub read_chunk_kb: u32,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            duration_secs: 10,
            size_mb: 64,
            read_chunk_kb: 512,
        }
    }
}

pub struct VideoHandle {
    pub id: PlayerId,
    pub video: iced_video_player::Video,
    pub pipeline: gst::Pipeline,
    pub state: PlayerState,
    last_seek_time: Option<Instant>,
    pending_seek_target: Option<Duration>,
}

impl VideoHandle {
    pub fn new(id: PlayerId, file_path: &str, name: &str) -> Result<Self, String> {
        Self::with_buffering(id, file_path, name, BufferConfig::default())
    }

    pub fn with_buffering(
        id: PlayerId,
        file_path: &str,
        name: &str,
        config: BufferConfig,
    ) -> Result<Self, String> {
        let uri = if file_path.starts_with("file://") {
            Url::parse(file_path).map_err(|e| e.to_string())?
        } else {
            Url::from_file_path(file_path).map_err(|_| "Failed to convert path to URL".to_string())?
        };

        gst::init().map_err(|e| e.to_string())?;

        let pipeline_str = format!(
            r#"playbin uri="{}" video-sink="videoscale ! videoconvert ! appsink name=iced_video drop=true caps=video/x-raw,format=NV12,pixel-aspect-ratio=1/1""#,
            uri.as_str()
        );

        let pipeline = gst::parse::launch(pipeline_str.as_ref())
            .map_err(|e| e.to_string())?
            .downcast::<gst::Pipeline>()
            .map_err(|_| "Failed to downcast to Pipeline".to_string())?;

        let dur_secs = config.duration_secs.max(2) as u64;
        let dur_ns = dur_secs * 1_000_000_000u64;
        let dur_ns_i64 = dur_ns as i64;
        let buf_size_bytes = (config.size_mb.max(16) as u32) * 1024 * 1024;
        let multiqueue_size_bytes = buf_size_bytes.max(128 * 1024 * 1024);
        let extra_size_bytes = multiqueue_size_bytes / 2;
        let extra_size_time = dur_ns / 2;
        let read_chunk_bytes = (config.read_chunk_kb.max(64) as u32) * 1024;

        pipeline.set_property("buffer-duration", dur_ns_i64);
        pipeline.set_property("buffer-size", buf_size_bytes as i32);
        pipeline.set_property("ring-buffer-max-size", buf_size_bytes as u64);

        pipeline.connect("source-setup", false, move |values| {
            if let Some(source) = values.get(1).and_then(|v| v.get::<gst::Element>().ok()) {
                if source.has_property("blocksize", None) {
                    // Set large read block size for smooth reads over SMB/NFS/WiFi mounts instead of 4KB default
                    let _ = source.set_property("blocksize", read_chunk_bytes);
                }
            }
            None
        });

        pipeline.connect("element-setup", false, move |values| {
            if let Some(elem) = values.get(1).and_then(|v| v.get::<gst::Element>().ok()) {
                let factory_name = elem.factory().map(|f| f.name().to_string()).unwrap_or_default();
                let name = elem.name();
                if factory_name == "decodebin" {
                    let _ = elem.set_property("max-size-buffers", 2000u32);
                    let _ = elem.set_property("max-size-time", dur_ns);
                    let _ = elem.set_property("max-size-bytes", multiqueue_size_bytes);
                } else if factory_name == "uridecodebin" {
                    let _ = elem.set_property("buffer-duration", dur_ns_i64);
                    let _ = elem.set_property("buffer-size", buf_size_bytes as i32);
                    let _ = elem.set_property("ring-buffer-max-size", buf_size_bytes as u64);
                } else if factory_name == "multiqueue" {
                    let _ = elem.set_property("use-interleave", false);
                    let _ = elem.set_property("max-size-buffers", 2000u32);
                    let _ = elem.set_property("extra-size-buffers", 1000u32);
                    let _ = elem.set_property("max-size-time", dur_ns);
                    let _ = elem.set_property("extra-size-time", extra_size_time);
                    let _ = elem.set_property("max-size-bytes", multiqueue_size_bytes);
                    let _ = elem.set_property("extra-size-bytes", extra_size_bytes);
                } else if name.as_str() == "vqueue" {
                    let _ = elem.set_property("max-size-buffers", 60u32);
                    let _ = elem.set_property("max-size-time", 1_000_000_000u64);
                    let _ = elem.set_property("max-size-bytes", 0u32);
                } else if name.as_str() == "aqueue" {
                    let _ = elem.set_property("max-size-time", 2_000_000_000u64);
                }
            }
            None
        });

        let video_sink: gst::Element = pipeline.property("video-sink");
        let pad = video_sink.pads().first().cloned().ok_or("No pads on video sink")?;
        let pad = pad.dynamic_cast::<gst::GhostPad>().map_err(|_| "Not a ghost pad")?;
        let bin = pad.parent_element().ok_or("No parent element")?.downcast::<gst::Bin>().map_err(|_| "Not a bin")?;
        let app_sink = bin.by_name("iced_video").ok_or("Could not find iced_video appsink")?;
        let app_sink = app_sink.downcast::<gst_app::AppSink>().map_err(|_| "Not an AppSink")?;

        let video = iced_video_player::Video::from_gst_pipeline(pipeline.clone(), app_sink, None)
            .map_err(|e| format!("iced_video_player error: {e:?}"))?;

        let mut handle = Self {
            id,
            video,
            pipeline,
            state: PlayerState::new(id, file_path.to_string(), name.to_string()),
            last_seek_time: None,
            pending_seek_target: None,
        };

        handle.set_volume(1.0);
        let dur = handle.duration();
        handle.state.duration = dur;

        Ok(handle)
    }

    pub fn set_volume(&mut self, volume: f64) {
        let clamped = volume.clamp(0.0, 1.0);
        self.state.volume = clamped;
        let effective_volume = if self.state.is_muted { 0.0 } else { clamped };
        self.pipeline.set_property("volume", effective_volume);
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.state.is_muted = muted;
        let effective_volume = if muted { 0.0 } else { self.state.volume };
        self.pipeline.set_property("volume", effective_volume);
    }

    pub fn toggle_play(&mut self) {
        if self.state.is_playing {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn play(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Playing);
        self.state.is_playing = true;
    }

    pub fn pause(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Paused);
        self.state.is_playing = false;
    }

    pub fn seek_random(&mut self) {
        let duration = self.duration();
        if duration > Duration::from_secs(2) {
            let max_secs = duration.as_secs_f64();
            let rand_secs = rand::thread_rng().gen_range(0.0..max_secs);
            self.seek(Duration::from_secs_f64(rand_secs));
        }
    }

    pub const STUCK_THRESHOLD_SECONDS: usize = 30;
    pub const SEEK_GRACE_PERIOD: Duration = Duration::from_secs(10);

    pub fn seek(&mut self, position: Duration) {
        self.seek_with_accuracy(position, true);
    }

    pub fn seek_with_accuracy(&mut self, position: Duration, accurate: bool) {
        let dur = self.duration();
        let margin = if dur > Duration::from_secs(2) {
            Duration::from_millis(800)
        } else {
            dur / 4
        };
        let clamped_pos = if dur > Duration::ZERO {
            position.min(dur.saturating_sub(margin))
        } else {
            position
        };

        self.state.position = clamped_pos;
        self.pending_seek_target = Some(clamped_pos);
        self.last_seek_time = Some(Instant::now());
        self.state.stuck_count = 0;
        self.state.last_checked_pos = clamped_pos;

        // Perform seek on iced_video_player::Video
        // For arrow-key scrubs (accurate = false), fast keyframe seek avoids decoding intermediate frames
        if let Err(e) = self.video.seek(clamped_pos, accurate) {
            log::warn!("Video::seek error ({:?}), falling back to pipeline.seek_simple", e);
            let pos_nanos = clamped_pos.as_nanos() as u64;
            let flags = gst::SeekFlags::FLUSH
                | if accurate {
                    gst::SeekFlags::ACCURATE
                } else {
                    gst::SeekFlags::empty()
                };
            let _ = self.pipeline.seek_simple(
                flags,
                gst::ClockTime::from_nseconds(pos_nanos),
            );
        }
    }

    pub fn seek_relative(&mut self, seconds: f64) {
        let current = self.position();
        let dur = self.duration();
        let margin = if dur > Duration::from_secs(2) {
            Duration::from_millis(800)
        } else {
            dur / 4
        };
        let new_pos = if seconds < 0.0 {
            current.saturating_sub(Duration::from_secs_f64(-seconds))
        } else {
            let target = current + Duration::from_secs_f64(seconds);
            if dur > Duration::ZERO && target >= dur {
                dur.saturating_sub(margin)
            } else {
                target
            }
        };
        // Use fast keyframe seek for smooth relative arrow-key scrubbing
        self.seek_with_accuracy(new_pos, false);
    }

    pub fn adjust_volume(&mut self, delta: f64) {
        self.set_volume(self.state.volume + delta);
    }

    fn query_pipeline_position(&self) -> Option<Duration> {
        self.pipeline
            .query_position::<gst::ClockTime>()
            .map(|t| Duration::from_nanos(t.nseconds()))
    }

    pub fn position(&self) -> Duration {
        if let Some(target) = self.pending_seek_target {
            if let Some(seek_time) = self.last_seek_time {
                let elapsed = seek_time.elapsed();
                if elapsed < Duration::from_secs(3) {
                    if let Some(pos) = self.query_pipeline_position() {
                        let diff = (pos.as_millis() as i64 - target.as_millis() as i64).abs();
                        if pos > Duration::ZERO && diff < 1200 {
                            return pos;
                        }
                    }
                    return target;
                }
            }
        }

        if let Some(pos) = self.query_pipeline_position() {
            if pos == Duration::ZERO && self.state.position > Duration::from_millis(500) {
                return self.state.position;
            }
            pos
        } else {
            self.state.position
        }
    }

    pub fn duration(&self) -> Duration {
        let video_dur = self.video.duration();
        if video_dur > Duration::ZERO {
            return video_dur;
        }
        self.pipeline
            .query_duration::<gst::ClockTime>()
            .map(|t| Duration::from_nanos(t.nseconds()))
            .unwrap_or(self.state.duration)
    }

    pub fn is_finished(&self) -> bool {
        // Direct EOS report from GStreamer bus via iced_video_player
        if self.video.eos() {
            return true;
        }

        // Never trigger finished while paused unless true EOS is reached
        if !self.state.is_playing || self.video.paused() {
            return false;
        }

        // Never trigger finished within 3 seconds of a seek
        if let Some(seek_time) = self.last_seek_time {
            if seek_time.elapsed() < Duration::from_secs(3) {
                return false;
            }
        }

        let dur = self.duration();
        let pos = self.position();
        dur > Duration::ZERO && pos >= dur.saturating_sub(Duration::from_millis(400))
    }

    /// Check if video playback is stuck (same position for too long while marked playing)
    pub fn check_stuck(&mut self) -> bool {
        // Smart check: never watchdog if playback is paused
        if !self.state.is_playing || self.video.paused() {
            self.state.stuck_count = 0;
            return false;
        }

        // Smart check: never watchdog during or immediately after a seek
        if let Some(seek_time) = self.last_seek_time {
            if seek_time.elapsed() < Self::SEEK_GRACE_PERIOD {
                self.state.stuck_count = 0;
                self.state.last_checked_pos = self.position();
                return false;
            }
        }

        let current_pos = self.position();
        if current_pos == self.state.last_checked_pos && self.duration() > Duration::ZERO {
            self.state.stuck_count += 1;
            if self.state.stuck_count >= Self::STUCK_THRESHOLD_SECONDS {
                return true;
            }
        } else {
            self.state.stuck_count = 0;
            self.state.last_checked_pos = current_pos;
        }
        false
    }
}
