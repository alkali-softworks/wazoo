/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Playback Transport, Seeking, A-B Looping & Watchdog
 *
 * Handles libmpv event dispatch, frame-update ticks, play/pause/stop, volume and
 * crossfade attenuation, seeking/frame-stepping, Mark-In/Out loops, and EOS/stuck detection.
 */

use super::handle::VideoHandle;
use super::mpv_client::{MpvEvent, MpvProperty};
use rand::Rng;
use std::time::{Duration, Instant};

impl VideoHandle {
    pub const STUCK_THRESHOLD_SECONDS: usize = 16;
    pub const SEEK_GRACE_PERIOD: Duration = Duration::from_secs(10);

    fn random_seek_target(dur: Duration) -> Duration {
        let max_secs = dur.as_secs_f64();
        let min_secs = (max_secs * 0.05).min(5.0);
        let max_bound = (max_secs * 0.85).max(min_secs);
        let rand_secs = if max_bound > min_secs {
            rand::thread_rng().gen_range(min_secs..max_bound)
        } else {
            0.0
        };
        Duration::from_secs_f64(rand_secs)
    }

    fn handle_mpv_event(&mut self, event: MpvEvent, needs_refresh_tracks: &mut bool) {
        match event {
            MpvEvent::StartFile => {
                self.is_eos = false;
                self.is_failed = false;
                self.file_loaded = false;
                self.cached_dwidth = None;
                self.cached_dheight = None;
                self.cached_eof_reached = false;
            }
            MpvEvent::Seek => {
                self.is_seeking = true;
                self.seek_restart_received = false;
            }
            MpvEvent::EndFile { is_error, is_eof } => {
                if is_error {
                    self.is_failed = true;
                    self.is_eos = true;
                } else if is_eof {
                    self.is_eos = true;
                    if !self.tracks_loaded && self.duration() == Duration::ZERO {
                        self.is_failed = true;
                    }
                }
            }
            MpvEvent::FileLoaded => {
                self.file_loaded = true;
                self.is_eos = false;
                self.is_failed = false;
                *needs_refresh_tracks = true;
                self.needs_render_dim_refresh = true;
            }
            MpvEvent::TracksOrVideoReconfig => {
                self.is_eos = false;
                self.is_failed = false;
                *needs_refresh_tracks = true;
                self.needs_render_dim_refresh = true;
            }
            MpvEvent::PlaybackRestart => {
                self.is_eos = false;
                self.is_failed = false;
                *needs_refresh_tracks = true;
                self.needs_render_dim_refresh = true;
                if self.is_seeking {
                    self.seek_restart_received = true;
                    if let Ok(mut frame) = self.frame.lock() {
                        frame.frame_seq = 0;
                    }
                }
            }
            MpvEvent::PropertyChange(prop) => match prop {
                MpvProperty::DWidth(w) => {
                    self.cached_dwidth = Some(w);
                    self.needs_render_dim_refresh = true;
                }
                MpvProperty::DHeight(h) => {
                    self.cached_dheight = Some(h);
                    self.needs_render_dim_refresh = true;
                }
                MpvProperty::Duration(dur) => {
                    self.state.duration = Duration::from_secs_f64(dur);
                }
                MpvProperty::TimePos(pos) => {
                    if self.pending_seek.is_none() {
                        let within_seek_grace = self
                            .last_seek_time
                            .map(|t| t.elapsed() < Duration::from_millis(1500))
                            .unwrap_or(false);
                        if !within_seek_grace {
                            self.state.position = Duration::from_secs_f64(pos);
                        }
                    }
                }
                MpvProperty::EofReached(flag) => {
                    self.cached_eof_reached = flag;
                }
                MpvProperty::PausedForCache(flag) => {
                    self.cached_paused_for_cache = flag;
                }
            },
            MpvEvent::Ignored => {}
        }
    }

    /// Update video frame if mpv has decoded a new presentation frame.
    pub fn update_frame(&mut self) -> bool {
        if self.mpv.is_none() {
            return false;
        }

        // Process pending mpv events with a safety limit to prevent UI thread lockups
        let mut event_count = 0;
        let mut needs_refresh_tracks = false;
        while event_count < 64 {
            event_count += 1;
            let event = match self.mpv.as_ref().and_then(|m| m.poll_event()) {
                Some(ev) => ev,
                None => break,
            };
            self.handle_mpv_event(event, &mut needs_refresh_tracks);
        }

        if self.is_seeking {
            if let Some(t) = self.last_seek_time {
                if t.elapsed() > Duration::from_millis(2000) {
                    self.is_seeking = false;
                    self.seek_restart_received = false;
                }
            }
        }

        if needs_refresh_tracks {
            self.refresh_audio_tracks();
            if self.file_loaded && self.state.duration == Duration::ZERO {
                if let Some(dur) = self
                    .mpv
                    .as_ref()
                    .and_then(|m| m.get_property_double("duration"))
                {
                    if dur > 0.0 && dur.is_finite() {
                        self.state.duration = Duration::from_secs_f64(dur);
                    }
                }
            }
        }

        if let Some(target) = self.pending_seek {
            if self.duration() > Duration::ZERO {
                self.pending_seek = None;
                self.last_seek_time = Some(Instant::now());
                self.is_seeking = true;
                self.seek_restart_received = false;
                self.clear_frame_black();
                if let Some(ref mpv) = self.mpv {
                    mpv.command_string(&format!(
                        "no-osd seek {:.3} absolute+exact",
                        target.as_secs_f64()
                    ));
                }
            }
        }

        if self.pending_seek_random {
            let dur = self.duration();
            if dur > Duration::from_secs(2) {
                self.pending_seek_random = false;
                self.last_seek_time = Some(Instant::now());
                let target = Self::random_seek_target(dur);
                self.seek_fast(target);
            }
        }

        self.check_and_apply_loop();

        self.render_pending_sw_frame()
    }

    fn apply_effective_volume(&self) {
        let effective_vol = (self.state.volume * self.crossfade_volume).clamp(0.0, 1.0);
        if let Some(ref mpv) = self.mpv {
            mpv.set_property_async_double("volume", effective_vol * 100.0);
        }
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.state.volume = volume.clamp(0.0, 1.0);
        self.apply_effective_volume();
    }

    /// Sets the dynamic crossfade attenuation multiplier (0.0 to 1.0) without altering
    /// the player's underlying user volume setting (`self.state.volume`).
    pub fn set_crossfade_volume(&mut self, factor: f64) {
        self.crossfade_volume = factor.clamp(0.0, 1.0);
        self.apply_effective_volume();
    }

    /// Resets the dynamic crossfade attenuation multiplier back to 1.0 (full audio output),
    /// restoring mpv playback volume to the player's configured `state.volume`.
    pub fn reset_crossfade_volume(&mut self) {
        self.set_crossfade_volume(1.0);
    }

    #[inline]
    pub fn crossfade_volume(&self) -> f64 {
        self.crossfade_volume
    }

    pub fn adjust_volume(&mut self, delta: f64) {
        self.set_volume(self.state.volume + delta);
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.state.is_muted = muted;
        if let Some(ref mpv) = self.mpv {
            mpv.set_property_async_flag("mute", muted);
        }
    }

    pub fn toggle_play(&mut self) {
        if self.state.is_playing {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn play(&mut self) {
        self.set_pause_internal(false);
    }

    pub fn pause(&mut self) {
        self.set_pause_internal(true);
    }

    pub fn stop(&mut self) {
        self.state.is_playing = false;
        if let Some(ref mpv) = self.mpv {
            let _ = mpv.command_async(&["stop"]);
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.set_pause_internal(paused);
    }

    pub fn is_playing(&self) -> bool {
        self.state.is_playing
    }

    fn set_pause_internal(&mut self, paused: bool) {
        self.state.is_playing = !paused;
        if let Some(ref mpv) = self.mpv {
            mpv.set_property_async_flag("pause", paused);
        }
    }

    pub fn set_speed(&mut self, speed: f64) {
        let clamped = speed.clamp(0.25, 4.0);
        if (self.speed - clamped).abs() < 0.001 {
            return;
        }
        self.speed = clamped;
        if let Some(ref mpv) = self.mpv {
            mpv.set_property_async_double("speed", clamped);
        }
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn is_seeking(&self) -> bool {
        self.is_seeking || self.pending_seek_random || self.pending_seek.is_some()
    }

    #[inline]
    pub fn pending_seek(&self) -> Option<Duration> {
        self.pending_seek
    }

    pub fn is_pending_seek_random(&self) -> bool {
        self.pending_seek_random
    }

    pub fn seek_fast(&mut self, position: Duration) {
        self.seek_internal(position.as_secs_f64(), false, false);
    }

    pub fn seek_random(&mut self) {
        self.is_seeking = true;
        self.seek_restart_received = false;
        self.clear_frame_black();
        let duration = self.duration();
        if duration > Duration::from_secs(2) {
            let target = Self::random_seek_target(duration);
            self.pending_seek_random = false;
            self.seek_fast(target);
        } else {
            self.pending_seek_random = true;
        }
    }

    pub fn seek(&mut self, position: Duration) {
        self.seek_internal(position.as_secs_f64(), false, true);
    }

    pub fn seek_relative(&mut self, seconds: f64) {
        self.seek_internal(seconds, true, false);
    }

    /// Pause playback and step forward by exactly one video frame.
    pub fn step_frame_forward(&mut self) {
        self.step_frame("frame-step");
    }

    /// Pause playback and step backward by exactly one video frame.
    pub fn step_frame_backward(&mut self) {
        self.step_frame("frame-back-step");
    }

    fn step_frame(&mut self, cmd: &str) {
        if self.mpv.is_none() {
            return;
        }
        self.pause();
        self.pending_seek = None;
        self.last_seek_time = None;
        if let Some(ref mpv) = self.mpv {
            mpv.command_string(cmd);
        }
        self.update_frame();
    }

    fn seek_internal(&mut self, val: f64, relative: bool, accurate: bool) {
        self.is_seeking = true;
        self.seek_restart_received = false;
        self.clear_frame_black();
        self.pending_seek_random = false;
        self.last_seek_time = Some(Instant::now());
        self.is_eos = false;
        self.state.stuck_count = 0;

        let current = self.position().as_secs_f64();
        let dur = self.duration().as_secs_f64();
        let max_target = if dur > 0.5 { dur - 0.1 } else { f64::MAX };
        let target = if relative {
            (current + val).clamp(0.0, max_target)
        } else {
            val.clamp(0.0, max_target)
        };
        self.state.position = Duration::from_secs_f64(target);

        if dur == 0.0 {
            self.pending_seek = Some(self.state.position);
        } else {
            self.pending_seek = None;
        }

        let mode = match (relative, accurate) {
            (true, true) => "relative+exact",
            (true, false) => "relative",
            (false, true) => "absolute+exact",
            (false, false) => "absolute",
        };

        if let Some(ref mpv) = self.mpv {
            mpv.command_string(&format!("no-osd seek {val} {mode}"));
        }
    }

    pub fn position(&self) -> Duration {
        if let Some(target) = self.pending_seek {
            return target;
        }
        self.state.position
    }

    pub fn duration(&self) -> Duration {
        self.state.duration
    }

    pub fn mark_in(&self) -> Option<Duration> {
        self.state.mark_in
    }

    pub fn mark_out(&self) -> Option<Duration> {
        self.state.mark_out
    }

    pub fn has_loop(&self) -> bool {
        self.state.mark_in.is_some() || self.state.mark_out.is_some()
    }

    pub fn set_mark_in(&mut self, pos: Option<Duration>) {
        self.state.mark_in = pos;
        if let (Some(in_pos), Some(out_pos)) = (self.state.mark_in, self.state.mark_out) {
            if in_pos >= out_pos {
                self.state.mark_out = None;
            }
        }
    }

    pub fn set_mark_out(&mut self, pos: Option<Duration>) {
        self.state.mark_out = pos;
        if let (Some(in_pos), Some(out_pos)) = (self.state.mark_in, self.state.mark_out) {
            if out_pos <= in_pos {
                self.state.mark_in = None;
            }
        }
    }

    pub fn clear_mark_in(&mut self) {
        self.state.mark_in = None;
    }

    pub fn clear_mark_out(&mut self) {
        self.state.mark_out = None;
    }

    pub fn clear_marks(&mut self) {
        self.state.mark_in = None;
        self.state.mark_out = None;
    }

    /// Check if the player has an active Mark-In / Mark-Out loop, and when the current playback
    /// position reaches or passes the loop boundary or end-of-stream, seek back to loop start.
    pub fn check_and_apply_loop(&mut self) -> bool {
        if !self.has_loop() {
            return false;
        }

        if let Some(seek_time) = self.last_seek_time {
            if seek_time.elapsed() < Duration::from_millis(300) {
                return false;
            }
        }

        let start = self.state.mark_in.unwrap_or(Duration::ZERO);

        let should_loop = if let Some(out_pos) = self.state.mark_out {
            self.is_eos || self.position() >= out_pos
        } else if self.state.mark_in.is_some() {
            let dur = self.duration();
            let pos = self.position();
            let near_end = dur > Duration::from_millis(500)
                && (pos >= dur || dur.saturating_sub(pos) <= Duration::from_millis(200));
            self.is_eos || near_end
        } else {
            false
        };

        if should_loop {
            self.is_eos = false;
            let was_playing = self.state.is_playing;
            self.seek(start);
            if was_playing {
                self.play();
            }
            return true;
        }

        false
    }

    /// Returns true if video playback encountered an unrecoverable decoding, demuxing, or format error.
    pub fn is_failed(&self) -> bool {
        self.is_failed
    }

    /// Returns true if video reached end of stream (EOS).
    pub fn is_eos(&self) -> bool {
        self.is_eos
    }

    pub fn is_finished(&self) -> bool {
        if self.is_failed {
            return true;
        }
        if self.has_loop() {
            return false;
        }
        if self.is_eos {
            return true;
        }
        if !self.tracks_loaded {
            return false;
        }
        let dur = self.duration();
        if dur < Duration::from_millis(500) {
            return false;
        }
        if self.cached_eof_reached {
            return true;
        }
        let pos = self.position();
        if pos >= dur || dur.saturating_sub(pos) <= Duration::from_millis(150) {
            if let Some(seek_time) = self.last_seek_time {
                if seek_time.elapsed() < Duration::from_millis(1500) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Check if video playback is stuck (same position for too long while marked playing).
    pub fn check_stuck(&mut self) -> bool {
        if self.is_failed {
            return true;
        }

        if !self.state.is_playing {
            self.state.stuck_count = 0;
            return false;
        }

        if self.has_loop() {
            self.check_and_apply_loop();
            return false;
        }

        if let Some(seek_time) = self.last_seek_time {
            if seek_time.elapsed() < Self::SEEK_GRACE_PERIOD {
                self.state.stuck_count = 0;
                self.state.last_checked_pos = self.position();
                return false;
            }
        }

        let current_pos = self.position();
        if current_pos == self.state.last_checked_pos {
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
