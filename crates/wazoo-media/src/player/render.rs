/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Frame Rendering, Dimensions & GPU Shader Controls
 *
 * Manages pixel/frame buffers, software frame transfers, video dimension and aspect-ratio
 * queries, and Iced GPU shader presentation (`view*` and CRT/wavy/fog filters).
 */

use super::handle::VideoHandle;
use crate::pipeline::FrameData;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

impl VideoHandle {
    /// Render the latest software-decoded frame into `self.frame` if mpv has an update ready.
    pub(super) fn render_pending_sw_frame(&mut self) -> bool {
        if self.mpv.is_none() {
            return false;
        }

        if self.needs_render_dim_refresh {
            self.check_update_render_dimensions();
            self.needs_render_dim_refresh = false;
        }

        if !self.mpv.as_ref().is_some_and(|m| m.has_frame_update()) {
            return false;
        }

        if self.render_width == 0 || self.render_height == 0 {
            self.check_update_render_dimensions();
            if self.render_width == 0 || self.render_height == 0 {
                return false;
            }
        }

        let rendered = self.mpv.as_ref().is_some_and(|mpv| {
            mpv.render_sw_frame(self.render_width, self.render_height, &mut self.pixel_buffer)
        });
        if rendered {
            {
                let mut frame_guard = self.frame.lock().unwrap();
                frame_guard.width = self.render_width;
                frame_guard.height = self.render_height;
                std::mem::swap(&mut frame_guard.pixels, &mut self.pixel_buffer);
                frame_guard.new_frame = true;
                frame_guard.frame_seq = frame_guard.frame_seq.wrapping_add(1);
            }

            if self.seek_restart_received {
                self.is_seeking = false;
                self.seek_restart_received = false;
            }

            self.state.position = self.position();
            let d = self.duration();
            if d > Duration::ZERO {
                self.state.duration = d;
            }
            return true;
        }

        false
    }

    /// Render video frame using a persistent GPU texture pipeline (flicker-free).
    pub fn view<'a, Message: 'a>(&'a self, opacity: f32) -> iced::Element<'a, Message> {
        self.view_with_fit(opacity, false)
    }

    /// Render video frame with custom fit mode (`fit_cover = true` for edge-to-edge ambient fill).
    pub fn view_with_fit<'a, Message: 'a>(
        &'a self,
        opacity: f32,
        fit_cover: bool,
    ) -> iced::Element<'a, Message> {
        self.view_full(
            opacity,
            fit_cover,
            self.crt_enabled,
            self.wavy_enabled,
            self.fog_enabled,
        )
    }

    /// Render video frame with custom fit and explicit GPU shader filter settings.
    pub fn view_full<'a, Message: 'a>(
        &'a self,
        opacity: f32,
        fit_cover: bool,
        crt_enabled: bool,
        wavy_enabled: bool,
        fog_enabled: bool,
    ) -> iced::Element<'a, Message> {
        let program = crate::pipeline::VideoProgram::new_full(
            self.id as u64,
            Arc::clone(&self.frame),
            Arc::clone(&self.alive),
            opacity,
            fit_cover,
            crt_enabled,
            wavy_enabled,
            fog_enabled,
            self.crt_settings,
        );
        iced::Element::new(crate::pipeline::video_shader(program))
    }

    pub fn set_crt_settings(&mut self, settings: wazoo_core::CrtSettings) {
        self.crt_settings = settings;
    }

    pub fn crt_settings(&self) -> wazoo_core::CrtSettings {
        self.crt_settings
    }

    pub fn set_crt_enabled(&mut self, enabled: bool) {
        self.crt_enabled = enabled;
    }

    pub fn crt_enabled(&self) -> bool {
        self.crt_enabled
    }

    pub fn set_wavy_enabled(&mut self, enabled: bool) {
        self.wavy_enabled = enabled;
    }

    pub fn wavy_enabled(&self) -> bool {
        self.wavy_enabled
    }

    pub fn set_fog_enabled(&mut self, enabled: bool) {
        self.fog_enabled = enabled;
    }

    pub fn fog_enabled(&self) -> bool {
        self.fog_enabled
    }

    pub fn set_filters(&mut self, crt: bool, wavy: bool, fog: bool) {
        self.crt_enabled = crt;
        self.wavy_enabled = wavy;
        self.fog_enabled = fog;
    }

    /// Overwrite pixel and frame buffers with zeroed bytes so no previous frame is visible.
    pub fn clear_frame_black(&mut self) {
        if let Ok(mut frame) = self.frame.lock() {
            frame.pixels.fill(0);
            frame.frame_seq = 0;
            frame.new_frame = true;
        }
        self.pixel_buffer.fill(0);
    }

    /// Overwrite pixel and frame buffers with solid opaque black (`[0, 0, 0, 255]`).
    pub(super) fn clear_frame_opaque_black(&mut self) {
        if let Ok(mut frame) = self.frame.lock() {
            for chunk in frame.pixels.chunks_exact_mut(4) {
                chunk[0] = 0;
                chunk[1] = 0;
                chunk[2] = 0;
                chunk[3] = 255;
            }
            frame.new_frame = true;
            frame.frame_seq = 0;
        }
        for chunk in self.pixel_buffer.chunks_exact_mut(4) {
            chunk[0] = 0;
            chunk[1] = 0;
            chunk[2] = 0;
            chunk[3] = 255;
        }
    }

    pub fn has_decoded_frame(&self) -> bool {
        if self.is_seeking() {
            return false;
        }
        if let Ok(guard) = self.frame.lock() {
            if guard.frame_seq == 0
                || guard.pixels.is_empty()
                || guard.width == 0
                || guard.height == 0
            {
                return false;
            }
            if guard.frame_seq >= 5 {
                return true;
            }
            Self::has_visual_content(&guard.pixels)
        } else {
            false
        }
    }

    /// Fast scan of a sample of pixels to verify that the buffer contains actual decoded video imagery
    /// rather than solid black `[0, 0, 0, 255]` or zeroed memory from initialization/loading.
    pub fn has_visual_content(pixels: &[u8]) -> bool {
        if pixels.is_empty() {
            return false;
        }
        let sample_stride = (pixels.len() / (128 * 4)).max(1) * 4;
        for i in (0..pixels.len()).step_by(sample_stride) {
            if i + 2 < pixels.len()
                && (pixels[i] > 2 || pixels[i + 1] > 2 || pixels[i + 2] > 2)
            {
                return true;
            }
        }
        false
    }

    pub fn frame_snapshot(&self) -> Option<Vec<u8>> {
        self.frame.lock().ok().map(|g| g.pixels.clone())
    }

    pub fn frame(&self) -> Arc<Mutex<FrameData>> {
        Arc::clone(&self.frame)
    }

    pub fn alive(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.alive)
    }

    pub fn video_dimensions(&self) -> Option<(u32, u32)> {
        if let (Some(w), Some(h)) = (self.cached_dwidth, self.cached_dheight) {
            if w > 0 && h > 0 {
                return Some((w, h));
            }
        }
        if self.mpv.is_none() || !self.file_loaded {
            return None;
        }
        for (w_prop, h_prop) in [
            ("dwidth", "dheight"),
            ("video-params/dw", "video-params/dh"),
            ("video-params/w", "video-params/h"),
        ] {
            if let (Some(w), Some(h)) = (
                self.get_property_i64(w_prop).filter(|&v| v > 0),
                self.get_property_i64(h_prop).filter(|&v| v > 0),
            ) {
                return Some((w as u32, h as u32));
            }
        }
        None
    }

    pub fn aspect_ratio(&self) -> Option<f32> {
        if let Some((w, h)) = self.video_dimensions() {
            if h > 0 {
                return Some(w as f32 / h as f32);
            }
        }
        if let Ok(guard) = self.frame.lock() {
            if guard.width > 0 && guard.height > 0 {
                return Some(guard.width as f32 / guard.height as f32);
            }
        }
        if self.mpv.is_none() || !self.file_loaded {
            return None;
        }
        if let Some(s) = self.get_property_string("video-params/aspect") {
            if let Ok(val) = s.parse::<f32>() {
                if val > 0.05 {
                    return Some(val);
                }
            }
        }
        None
    }

    pub(super) fn check_update_render_dimensions(&mut self) {
        if let Some((w, h)) = self.video_dimensions() {
            if w > 0 && h > 0 {
                let max_dim = 1280.0f32;
                let scale = (max_dim / (w as f32).max(h as f32)).min(1.0);
                let target_w = (((w as f32 * scale).round() as u32).max(16) / 2) * 2;
                let target_h = (((h as f32 * scale).round() as u32).max(16) / 2) * 2;
                if self.render_width != target_w || self.render_height != target_h {
                    self.render_width = target_w;
                    self.render_height = target_h;
                    let buf_size = (target_w * target_h * 4) as usize;
                    self.pixel_buffer.resize(buf_size, 0);
                }
            }
        }
    }
}
