use std::time::Duration;
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

pub struct VideoHandle {
    pub id: PlayerId,
    pub video: iced_video_player::Video,
    pub pipeline: gst::Pipeline,
    pub state: PlayerState,
}

impl VideoHandle {
    pub fn new(id: PlayerId, file_path: &str, name: &str) -> Result<Self, String> {
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
        };

        handle.set_volume(1.0);
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

    pub fn seek(&mut self, position: Duration) {
        let pos_nanos = position.as_nanos() as u64;
        let _ = self.pipeline.seek_simple(
            gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
            gst::ClockTime::from_nseconds(pos_nanos),
        );
        self.state.position = position;
    }

    pub fn seek_relative(&mut self, seconds: f64) {
        let current = self.position();
        let new_pos = if seconds < 0.0 {
            current.saturating_sub(Duration::from_secs_f64(-seconds))
        } else {
            let dur = self.duration();
            let target = current + Duration::from_secs_f64(seconds);
            if dur > Duration::ZERO && target > dur {
                dur
            } else {
                target
            }
        };
        self.seek(new_pos);
    }

    pub fn adjust_volume(&mut self, delta: f64) {
        self.set_volume(self.state.volume + delta);
    }

    pub fn position(&self) -> Duration {
        self.pipeline
            .query_position::<gst::ClockTime>()
            .map(|t| Duration::from_nanos(t.nseconds()))
            .unwrap_or(self.state.position)
    }

    pub fn duration(&self) -> Duration {
        self.pipeline
            .query_duration::<gst::ClockTime>()
            .map(|t| Duration::from_nanos(t.nseconds()))
            .unwrap_or(self.state.duration)
    }

    pub fn is_finished(&self) -> bool {
        let dur = self.duration();
        let pos = self.position();
        dur > Duration::ZERO && pos >= dur.saturating_sub(Duration::from_millis(300))
    }

    /// Check if video playback is stuck (same position for too long while marked playing)
    pub fn check_stuck(&mut self) -> bool {
        if !self.state.is_playing {
            return false;
        }

        let current_pos = self.position();
        if current_pos == self.state.last_checked_pos && self.duration() > Duration::ZERO {
            self.state.stuck_count += 1;
            if self.state.stuck_count >= 3 {
                return true;
            }
        } else {
            self.state.stuck_count = 0;
            self.state.last_checked_pos = current_pos;
        }
        false
    }
}
