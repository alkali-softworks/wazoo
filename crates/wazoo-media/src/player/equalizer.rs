/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Equalizer & Color Filter Controls
 *
 * Builds FFmpeg `eq` filter strings for software rendering and synchronizes
 * gamma, contrast, brightness, and saturation properties with libmpv.
 */

use super::handle::VideoHandle;

impl VideoHandle {
    /// Construct FFmpeg `eq` video filter string for software rendering equalizer adjustments.
    /// Returns an empty string if all equalizer parameters are at their defaults (0.0).
    pub fn build_eq_filter_string(
        gamma: f64,
        contrast: f64,
        brightness: f64,
        saturation: f64,
    ) -> String {
        if gamma.abs() < 0.001
            && contrast.abs() < 0.001
            && brightness.abs() < 0.001
            && saturation.abs() < 0.001
        {
            return String::new();
        }

        // Map UI slider values (-100.0..=100.0) to FFmpeg eq filter parameters:
        // gamma: default 1.0, range 0.1..10.0 (reciprocal symmetry around 1.0)
        let eq_gamma = if gamma >= 0.0 {
            1.0 + (gamma / 100.0) * 1.5
        } else {
            1.0 / (1.0 + (-gamma / 100.0) * 1.5)
        };

        // contrast: default 1.0, range 0.0..2.5
        let eq_contrast = if contrast >= 0.0 {
            1.0 + (contrast / 100.0) * 1.5
        } else {
            (1.0 + (contrast / 100.0)).max(0.0)
        };

        // brightness: default 0.0, range -0.8..0.8
        let eq_brightness = (brightness / 100.0) * 0.8;

        // saturation: default 1.0, range 0.0..2.5
        let eq_saturation = if saturation >= 0.0 {
            1.0 + (saturation / 100.0) * 1.5
        } else {
            (1.0 + (saturation / 100.0)).max(0.0)
        };

        format!(
            "eq=gamma={eq_gamma:.3}:contrast={eq_contrast:.3}:brightness={eq_brightness:.3}:saturation={eq_saturation:.3}"
        )
    }

    /// Apply current equalizer parameters to libmpv's `vf` filter chain and VO properties.
    pub fn update_equalizer(&mut self) {
        let filter = Self::build_eq_filter_string(
            self.gamma,
            self.contrast,
            self.brightness,
            self.saturation,
        );
        self.set_property_string("vf", &filter);

        // Also update standard VO properties for compatibility if hardware VO is used
        if let Some(ref mpv) = self.mpv {
            for (name, val) in [
                ("gamma", self.gamma),
                ("contrast", self.contrast),
                ("brightness", self.brightness),
                ("saturation", self.saturation),
            ] {
                mpv.set_property_double(name, val.clamp(-100.0, 100.0));
            }

            // If the video is paused, trigger a zero-seek so mpv immediately re-decodes the current paused frame with new filter settings
            if !self.state.is_playing {
                mpv.command_string("no-osd seek 0 relative exact");
            }
        }
    }

    pub fn set_equalizer(&mut self, gamma: f64, contrast: f64, brightness: f64, saturation: f64) {
        let g = gamma.clamp(-100.0, 100.0);
        let c = contrast.clamp(-100.0, 100.0);
        let b = brightness.clamp(-100.0, 100.0);
        let s = saturation.clamp(-100.0, 100.0);
        if (self.gamma - g).abs() < 0.001
            && (self.contrast - c).abs() < 0.001
            && (self.brightness - b).abs() < 0.001
            && (self.saturation - s).abs() < 0.001
        {
            return;
        }
        self.gamma = g;
        self.contrast = c;
        self.brightness = b;
        self.saturation = s;
        self.update_equalizer();
    }

    fn set_eq_value(slot: &mut f64, value: f64) -> bool {
        let clamped = value.clamp(-100.0, 100.0);
        if (*slot - clamped).abs() < 0.001 {
            return false;
        }
        *slot = clamped;
        true
    }

    pub fn set_gamma(&mut self, gamma: f64) {
        if Self::set_eq_value(&mut self.gamma, gamma) {
            self.update_equalizer();
        }
    }

    pub fn set_contrast(&mut self, contrast: f64) {
        if Self::set_eq_value(&mut self.contrast, contrast) {
            self.update_equalizer();
        }
    }

    pub fn set_brightness(&mut self, brightness: f64) {
        if Self::set_eq_value(&mut self.brightness, brightness) {
            self.update_equalizer();
        }
    }

    pub fn set_saturation(&mut self, saturation: f64) {
        if Self::set_eq_value(&mut self.saturation, saturation) {
            self.update_equalizer();
        }
    }

    pub fn gamma(&self) -> f64 {
        self.gamma
    }

    pub fn contrast(&self) -> f64 {
        self.contrast
    }

    pub fn brightness(&self) -> f64 {
        self.brightness
    }

    pub fn saturation(&self) -> f64 {
        self.saturation
    }
}
