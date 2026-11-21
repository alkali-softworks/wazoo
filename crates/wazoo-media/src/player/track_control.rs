/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Audio & Subtitle Track Selection and Language Preferences
 *
 * Queries libmpv's `track-list`, applies preferred audio/subtitle language matching
 * and i18n fallbacks, and handles manual track switching.
 */

use super::handle::VideoHandle;
use super::tracks::{
    AudioTrack, SubtitleTrack, build_alang_string, build_slang_string,
    build_slang_string_with_fallback, find_matching_audio_track,
    get_subtitle_track_preference_string, get_track_preference_string, is_signs_or_songs_track,
    select_best_subtitle_track_with_fallback, subtitle_track_matches_preference,
    track_matches_preference,
};

impl VideoHandle {
    pub fn set_subtitles_visible(&mut self, visible: bool) {
        if let Some(ref mpv) = self.mpv {
            mpv.set_property_async_flag("sub-visibility", visible);
        }
    }

    pub fn audio_tracks(&self) -> &[AudioTrack] {
        &self.state.audio_tracks
    }

    pub fn current_audio_track_id(&self) -> Option<i64> {
        self.state.current_audio_track_id
    }

    pub fn set_audio_track(&mut self, track_id: i64) {
        let Some(ref mpv) = self.mpv else {
            return;
        };
        mpv.set_property_i64("aid", track_id);
        mpv.set_property_string("aid", &track_id.to_string());
        self.state.current_audio_track_id = Some(track_id);
        for t in &mut self.state.audio_tracks {
            t.is_selected = t.id == track_id;
        }
        if let Some(track) = self.state.audio_tracks.iter().find(|t| t.id == track_id) {
            let pref = get_track_preference_string(track);
            if !pref.starts_with("Track ") {
                self.preferred_audio_language = Some(pref.clone());
                let alang = build_alang_string(&pref);
                if !alang.is_empty() {
                    mpv.set_property_string("alang", &alang);
                }
            }
        }
    }

    pub fn subtitle_tracks(&self) -> &[SubtitleTrack] {
        &self.state.subtitle_tracks
    }

    pub fn current_subtitle_track_id(&self) -> Option<i64> {
        self.state.current_subtitle_track_id
    }

    pub fn set_subtitle_track(&mut self, track_id: i64) {
        let Some(ref mpv) = self.mpv else {
            return;
        };
        mpv.set_property_i64("sid", track_id);
        mpv.set_property_string("sid", &track_id.to_string());
        self.state.current_subtitle_track_id = Some(track_id);
        for t in &mut self.state.subtitle_tracks {
            t.is_selected = t.id == track_id;
        }
        if let Some(track) = self.state.subtitle_tracks.iter().find(|t| t.id == track_id) {
            let pref = get_subtitle_track_preference_string(track);
            if !pref.starts_with("Track ") {
                self.preferred_subtitle_language = Some(pref.clone());
                let slang = build_slang_string(&pref);
                if !slang.is_empty() {
                    mpv.set_property_string("slang", &slang);
                }
            }
        }
        self.set_subtitles_visible(true);
        if let Some(ref mpv) = self.mpv {
            mpv.command_string("no-osd seek 0 relative+exact");
        }
    }

    /// Loads an external subtitle file (`.srt`, `.vtt`, `.ass`, `.ssa`) into the active player
    /// and selects it as the active subtitle track, returning its index in `subtitle_tracks()`.
    pub fn add_external_subtitle(&mut self, sub_path: &str) -> Result<usize, String> {
        let trimmed = sub_path.trim();
        if trimmed.is_empty() {
            return Err("Subtitle path is empty".to_string());
        }

        if let Some(existing_idx) = self
            .state
            .subtitle_tracks
            .iter()
            .position(|t| t.external_filename.as_deref() == Some(trimmed))
        {
            let track_id = self.state.subtitle_tracks[existing_idx].id;
            if self.mpv.is_some() {
                self.set_subtitle_track(track_id);
            } else {
                for (i, t) in self.state.subtitle_tracks.iter_mut().enumerate() {
                    t.is_selected = i == existing_idx;
                }
                self.state.current_subtitle_track_id = Some(track_id);
            }
            return Ok(existing_idx);
        }

        if let Some(ref mpv) = self.mpv {
            let res = mpv.command(&["sub-add", trimmed, "select"])?;
            if res < 0 {
                return Err(format!(
                    "Failed to load subtitle file: {trimmed} (mpv error {res})"
                ));
            }
            let (audio_tracks, mut sub_tracks) = self.collect_track_list();
            let idx = sub_tracks
                .iter()
                .position(|t| t.external_filename.as_deref() == Some(trimmed))
                .or_else(|| sub_tracks.iter().position(|t| t.is_selected))
                .or_else(|| sub_tracks.len().checked_sub(1))
                .ok_or_else(|| format!("Subtitle track did not register in mpv: {trimmed}"))?;
            let selected_id = sub_tracks[idx].id;
            for t in &mut sub_tracks {
                t.is_selected = t.id == selected_id;
            }
            self.state.audio_tracks = audio_tracks;
            self.state.subtitle_tracks = sub_tracks;
            self.state.current_subtitle_track_id = Some(selected_id);
            self.tracks_loaded = true;
            self.set_subtitles_visible(true);
            if let Some(ref mpv) = self.mpv {
                mpv.command_string("no-osd seek 0 relative+exact");
            }
            Ok(idx)
        } else {
            let new_id = self
                .state
                .subtitle_tracks
                .iter()
                .map(|t| t.id)
                .max()
                .unwrap_or(0)
                + 1;
            let path_obj = std::path::Path::new(trimmed);
            let title = path_obj
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());
            let codec = path_obj
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_ascii_lowercase());
            for t in &mut self.state.subtitle_tracks {
                t.is_selected = false;
            }
            self.state.subtitle_tracks.push(SubtitleTrack {
                id: new_id,
                title,
                lang: None,
                codec,
                is_selected: true,
                is_default: false,
                is_forced: false,
                external_filename: Some(trimmed.to_string()),
                ff_index: None,
            });
            self.state.current_subtitle_track_id = Some(new_id);
            Ok(self.state.subtitle_tracks.len() - 1)
        }
    }

    fn collect_track_list(&self) -> (Vec<AudioTrack>, Vec<SubtitleTrack>) {
        let count = self.get_property_i64("track-list/count").unwrap_or(0);
        let mut audio_tracks = Vec::new();
        let mut sub_tracks = Vec::new();

        for i in 0..count {
            let track_type = self.get_property_string(&format!("track-list/{i}/type"));
            let id = self
                .get_property_i64(&format!("track-list/{i}/id"))
                .unwrap_or(0);
            let title = self.get_property_string(&format!("track-list/{i}/title"));
            let lang = self.get_property_string(&format!("track-list/{i}/lang"));
            let codec = self.get_property_string(&format!("track-list/{i}/codec"));
            let selected = self
                .get_property_bool(&format!("track-list/{i}/selected"))
                .unwrap_or(false);

            if track_type.as_deref() == Some("audio") {
                audio_tracks.push(AudioTrack {
                    id,
                    title,
                    lang,
                    codec,
                    is_selected: selected,
                });
            } else if track_type.as_deref() == Some("sub") {
                let external_filename =
                    self.get_property_string(&format!("track-list/{i}/external-filename"));
                let ff_index = self.get_property_i64(&format!("track-list/{i}/ff-index"));
                let default = self
                    .get_property_bool(&format!("track-list/{i}/default"))
                    .unwrap_or(false);
                let forced = self
                    .get_property_bool(&format!("track-list/{i}/forced"))
                    .unwrap_or(false);
                sub_tracks.push(SubtitleTrack {
                    id,
                    title,
                    lang,
                    codec,
                    is_selected: selected,
                    is_default: default,
                    is_forced: forced,
                    external_filename,
                    ff_index,
                });
            }
        }

        (audio_tracks, sub_tracks)
    }

    fn resolve_audio_tracks(&mut self, mut audio_tracks: Vec<AudioTrack>) {
        // Query mpv for the currently active audio track.
        // Important: mpv's "aid" property could be set to "no", "auto", or an ID from a previous/different file.
        // We only consider an aid from mpv valid if it actually exists in this file's audio_tracks list.
        let mpv_selected_aid = self
            .get_property_i64("aid")
            .or_else(|| {
                self.get_property_string("aid")
                    .as_deref()
                    .and_then(|s| s.parse::<i64>().ok())
            })
            .filter(|id| audio_tracks.iter().any(|t| t.id == *id))
            .or_else(|| audio_tracks.iter().find(|t| t.is_selected).map(|t| t.id));

        let mut current_aid = mpv_selected_aid.or(self.state.current_audio_track_id);

        // Auto-select preferred audio track on initial load of the file tracks
        if !self.tracks_loaded && !audio_tracks.is_empty() {
            let mut target_aid = None;
            if let Some(ref pref) = self.preferred_audio_language {
                if let Some(matching_id) = find_matching_audio_track(&audio_tracks, pref) {
                    target_aid = Some(matching_id);
                }
            }

            // Fallback: if no preference matched (or no preference configured) and mpv has no valid track selected,
            // default to the container's default audio track or the first available audio track.
            if target_aid.is_none() && current_aid.is_none() {
                target_aid = audio_tracks
                    .iter()
                    .find(|t| t.is_selected)
                    .map(|t| t.id)
                    .or_else(|| audio_tracks.first().map(|t| t.id));
            }

            if let Some(desired_id) = target_aid {
                if current_aid != Some(desired_id) {
                    self.set_audio_track(desired_id);
                    current_aid = Some(desired_id);
                }
            }
        }

        if let Some(aid) = current_aid {
            for t in &mut audio_tracks {
                t.is_selected = t.id == aid;
            }
        }

        self.state.audio_tracks = audio_tracks;
        self.state.current_audio_track_id = current_aid;
    }

    fn resolve_subtitle_tracks(&mut self, mut sub_tracks: Vec<SubtitleTrack>) -> Option<String> {
        let is_sub_initial_load =
            !self.tracks_loaded || self.state.current_subtitle_track_id.is_none();
        let sid_i64 = self.get_property_i64("sid");
        let sid_str = self.get_property_string("sid");
        let mpv_selected_sid = sid_i64
            .or_else(|| sid_str.as_deref().and_then(|s| s.parse::<i64>().ok()))
            .filter(|id| sub_tracks.iter().any(|t| t.id == *id))
            .or_else(|| sub_tracks.iter().find(|t| t.is_selected).map(|t| t.id));

        let mut current_sid = mpv_selected_sid.or(self.state.current_subtitle_track_id);

        if is_sub_initial_load && !sub_tracks.is_empty() {
            let pref = self.preferred_subtitle_language.as_deref();
            let i18n_lang = self.i18n_language.as_deref();
            let effective_lang = self.effective_subtitle_language();

            let mut target_sid = None;

            if let Some(sid) = mpv_selected_sid {
                let active_track = sub_tracks.iter().find(|t| t.id == sid);
                let is_signs = active_track.map_or(false, is_signs_or_songs_track);
                let matches_pref = active_track.map_or(false, |t| {
                    subtitle_track_matches_preference(t, effective_lang)
                });

                // If mpv selected a Signs/Songs track, or a track that does not match the effective language
                // while an explicit preferred or i18n language match is available, select the matching track instead.
                if is_signs || !matches_pref {
                    if let Some(best) =
                        select_best_subtitle_track_with_fallback(&sub_tracks, pref, i18n_lang)
                    {
                        if best != sid {
                            target_sid = Some(best);
                        }
                    }
                }
            } else if sid_str.as_deref() == Some("no") || current_sid.is_some() {
                // mpv defaulted to no track (e.g. MKV container tracks with disposition 'default: 0')
                // or has no valid track selected. Select the best track according to preference/defaults.
                target_sid = select_best_subtitle_track_with_fallback(&sub_tracks, pref, i18n_lang);
            }

            if let Some(desired_id) = target_sid {
                if current_sid != Some(desired_id) {
                    log::info!(
                        "Auto-selecting subtitle track (id {}) on initial load (effective preference: {})",
                        desired_id,
                        effective_lang
                    );
                    self.set_subtitle_track(desired_id);
                    current_sid = Some(desired_id);
                }
            }
        }

        if let Some(sid) = current_sid {
            for t in &mut sub_tracks {
                t.is_selected = t.id == sid;
            }
        }

        self.state.subtitle_tracks = sub_tracks;
        self.state.current_subtitle_track_id = current_sid;

        sid_str
    }

    pub fn refresh_audio_tracks(&mut self) {
        if self.mpv.is_none() {
            return;
        }
        let (audio_tracks, sub_tracks) = self.collect_track_list();
        self.resolve_audio_tracks(audio_tracks);
        let sid_str = self.resolve_subtitle_tracks(sub_tracks);

        if !self.state.audio_tracks.is_empty()
            || (!self.state.subtitle_tracks.is_empty()
                && (self.state.current_subtitle_track_id.is_some()
                    || sid_str.as_deref() == Some("no")))
        {
            self.tracks_loaded = true;
        }
    }

    /// Returns the preferred subtitle language if known; otherwise falls back to the i18n language setting
    /// (defaulting to `"en"` if neither is known).
    pub fn effective_subtitle_language(&self) -> &str {
        self.preferred_subtitle_language
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| self.i18n_language.as_deref().filter(|s| !s.trim().is_empty()))
            .unwrap_or("en")
    }

    pub fn set_preferred_subtitle_language(&mut self, pref: Option<String>) {
        self.preferred_subtitle_language = pref;
        let slang = build_slang_string_with_fallback(
            self.preferred_subtitle_language.as_deref(),
            self.i18n_language.as_deref(),
        );
        if !slang.is_empty() {
            self.set_property_string("slang", &slang);
        }
    }

    pub fn preferred_subtitle_language(&self) -> Option<&str> {
        self.preferred_subtitle_language.as_deref()
    }

    pub fn set_i18n_language(&mut self, lang: Option<String>) {
        self.i18n_language = lang;
        if self.preferred_subtitle_language.is_none() {
            let slang = build_slang_string_with_fallback(None, self.i18n_language.as_deref());
            if !slang.is_empty() {
                self.set_property_string("slang", &slang);
            }
        }
    }

    pub fn i18n_language(&self) -> Option<&str> {
        self.i18n_language.as_deref()
    }

    pub fn set_preferred_audio_language(&mut self, pref: Option<String>) {
        self.preferred_audio_language = pref.clone();
        if let Some(ref p) = pref {
            let alang = build_alang_string(p);
            if !alang.is_empty() {
                self.set_property_string("alang", &alang);
            }
            let current_already_matches = self.state.current_audio_track_id.map_or(false, |aid| {
                self.state
                    .audio_tracks
                    .iter()
                    .any(|t| t.id == aid && track_matches_preference(t, p))
            });
            if !current_already_matches {
                if let Some(matching_id) = find_matching_audio_track(&self.state.audio_tracks, p) {
                    if self.state.current_audio_track_id != Some(matching_id) {
                        self.set_audio_track(matching_id);
                    }
                }
            }
        }
    }

    pub fn preferred_audio_language(&self) -> Option<&str> {
        self.preferred_audio_language.as_deref()
    }
}
