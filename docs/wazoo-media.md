# `wazoo-media` Subsystem Architecture

The [`wazoo-media`] crate is the media playback and presentation engine of Wazoo. It bridges native `libmpv` decoding to Iced's WGPU rendering pipeline, provides continuous vertical scroll layout calculations, and parses interactive subtitle cues.

---

## 📁 Module Organization

```
crates/wazoo-media/src/
├── cube.rs           # 3D bouncing video cube pipeline, projection math, and sheen shaders
├── lib.rs            # Crate root and high-level re-exports
├── mpv_ffi.rs        # Low-level C FFI bindings & dynamic loader for libmpv
├── pipeline.rs       # Custom WGPU multi-shader pipeline (CRT, Wavy, Fog) and texture cache
├── player/           # High-level video playback handles & track controls
│   ├── handle.rs     # VideoHandle implementation (libmpv lifecycle, frames, playback)
│   ├── mod.rs        # Player module root
│   ├── state.rs      # PlayerState and BufferConfig structures
│   └── tracks.rs     # Audio and Subtitle track management & preference matching
├── scroll.rs         # Continuous scroll feed physics and volume falloff engine
└── subtitles.rs      # Subtitle transcript parsing (.srt, .ass) & FFmpeg extraction
```

---

## 🎬 `libmpv` FFI Bridge (`mpv_ffi.rs`)

The media engine wraps `libmpv` directly via C FFI.

### Dynamic Symbol Resolution

To provide maximum portability across operating systems and distribution packaging:
1. `mpv_ffi` first attempts to dynamically load `libmpv` at runtime:
   - **Linux:** `libmpv.so.2`, `libmpv.so` (beside executable or `/usr/lib`)
   - **macOS:** `libmpv.2.dylib`, `libmpv.dylib` (beside executable, Frameworks, or Homebrew)
   - **Windows:** Embedded inside the portable binary, or loaded from `libmpv-2.dll` / `mpv-2.dll` beside the executable, user-isolated AppData, or system path. Current working directory (CWD) is excluded in release builds to prevent binary planting / DLL hijacking.
2. If dynamic loading fails, it falls back to linked C symbols resolved at build time.

### `libmpv` Configuration & Hardening

Each player instance is initialized with options tuned for multi-window ambient playback:
- `vo=libmpv`: Uses the software rendering context rather than native OS window handles.
- `hwdec`: Configurable via the `WAZOO_HWDEC` environment variable (defaults to software decoding `no` to eliminate GPU driver deadlocks when windows are occluded or behind other apps).
- `demuxer-readahead-secs` & `demuxer-max-bytes`: Configured by [`BufferConfig`] to keep memory usage bounded across multiple concurrent players.
- `force-seekable=yes` & `hr-seek-framedrop=yes`: Ensures ultra-responsive scrubbing and random seeking.
- `Security Hardening`: In release builds on Windows, DLL resolution is strictly restricted to trusted executable, user AppData, and system paths. Subtitle extraction processes strictly enforce protocol sandboxing (`-protocol_whitelist file,crypto`), validate stream mappings, and protect against leading-hyphen argument confusion.

---

## 🖼️ WGPU Presentation Pipeline (`pipeline.rs`)

Rather than letting `libmpv` draw directly to an OS window, Wazoo renders frames into GPU textures inside Iced's WGPU pipeline via [`VideoPipeline`].

```mermaid
graph LR
    mpv["libmpv Render Context"] -->|BGR0 / BGRA32| FrameBuf["FrameData<br/>(Arc<Mutex<FrameData>>)"]
    FrameBuf -->|queue.write_texture| Texture["wgpu::Texture"]
    Texture --> BindGroup["wgpu::BindGroup"]
    Uniforms["Scale, Opacity, Time & Filter Uniforms"] --> BindGroup
    BindGroup --> Shader["WGSL Multi-Shader Pipeline<br/>(CRT, Wavy, Fog)"]
    Shader --> Framebuffer["Iced Viewport Quad"]
```

### Advanced GPU Shader Pipeline Features:
1. **Multi-Player Scalability**: Renders 1 to 12 simultaneous video players within a single native window without multiple X11/Wayland sub-windows.
2. **Dynamic Window Opacity**: Opacity transitions and transparency are applied directly in the fragment shader without OS compositor glitches.
3. **Zero-Copy Video Swapchains**: Only textures that received a new frame on the tick are updated on the GPU via `queue.write_texture`.
4. **Procedural CRT Emulation**: Fragment shader simulates spherical tube curvature distortion, phosphorescent bloom glow, RGB subpixel chromatic aberration, scanlines, analog RF noise, and radial vignette falloff.
5. **Wavy Fluid Displacement**: Real-time sinusoidal UV coordinates distortion producing animated liquid surface waves.
6. **Volumetric Fog**: Multi-octave fractional Brownian motion (fBm) noise drifting procedurally across the screen.
7. **Pillarbox/Letterbox UV Clamping**: Automatically detects content aspect ratio and clamps UV coordinates to valid active frame boundaries, eliminating edge-smear artifacts when playing non-16:9 videos.
8. **Shader-Filtered TV Static**: Dynamic noise frame buffers generated during buffering are routed through the same WGPU shader pipeline (`0x8000_0000 | player_id`), ensuring all post-processing effects apply authentically to loading screens.

---

## 🎲 3D Video Cube Pipeline (`cube.rs`)

The [`cube.rs`] module provides real-time 3D projected video cube rendering inside Iced:

- **6-Face UV Texture Mapping**: Projects live decoded video frames across all six cube faces with directional lighting.
- **Dynamic Specular Sheen**: GPU fragment shader sweeps a glowing light sheen across the cube surfaces based on configurable speed, width, intensity, and angle.
- **Interactive Focus & Collision Physics**: Each cube simulates 3D Euler angle rotation, linear velocity, boundary bounce elasticity, and flashes an emerald primary glow border when focused.
- **Dual Presentation Modes**: Supports in-app 3D canvas rendering (Mode 8) and transparent borderless desktop overlay screensavers (Mode 9).

---

## 🎮 Video Player Handle (`player/handle.rs`)

The [`VideoHandle`] struct provides the ergonomic, safe Rust API for managing an active video stream:

### Key Features:
- **Playback Controls**: `play()`, `pause()`, `toggle_pause()`, `seek_relative(seconds)`, `seek_percent(pct)`, `seek_random()`.
- **A-B Looping**: `set_mark_in()`, `set_mark_out()`, `clear_mark_in()`, `clear_mark_out()`. When active, `libmpv` automatically loops between marked timestamps.
- **Software Video Equalizer**: Constructs FFmpeg `eq` video filter strings dynamically for `gamma`, `contrast`, `brightness`, and `saturation`.
- **Watchdog Protection**: `check_stuck()` detects when a player is supposed to be playing but has not advanced its playback timestamp for 30 seconds (`STUCK_THRESHOLD_SECONDS`), triggering an automatic recovery or advance.
- **Non-Blocking Handle Creation**: Video handles avoid synchronous `mpv_get_property` calls during initialization, populating default dimensions (`1280x720`) and duration `0` immediately so the UI thread never freezes on slow mounts while `libmpv` reads container headers. Real dimensions and durations resolve asynchronously via `update_frame()`.
- **Asynchronous Lifecycle & Background Teardown**: `VideoHandle::drop()` halts audio and video immediately via non-blocking `mpv_command_async("stop")` and offloads `mpv_render_context_free` and `mpv_terminate_destroy` to a dedicated `wazoo-mpv-teardown` background thread. This eliminates UI freezes while `libmpv` joins demuxer worker threads during rapid navigation.
- **Token-Boundary Track Matching (`player/tracks.rs`)**: `matches_alias_token()` matches short 2-character language codes (`en`, `es`) at discrete word boundaries rather than arbitrary substrings, preventing false positives (e.g. matching `Clean Audio` or `French` as English). Untagged tracks preserve container stream IDs (`Track 1`, `Track 2`), and generic numbers are guarded from overwriting global language preferences.

---

## 🌊 Continuous Scroll Engine (`scroll.rs`)

The [`ScrollEngine`] powers the infinite vertical video feed (Scroll Mode):

### 1. Aspect-Ratio Dependent Layout

Videos are not forced into rigid boxes. The layout engine calculates each tile's real display height using its native aspect ratio:

$$\text{Height} = \frac{\text{Window Width}}{\text{Aspect Ratio}}$$

### 2. Lookahead Margins & Preloading

To ensure smooth scrolling without blank frames:
- `needs_new_player_with_margin(margin)` checks if the bottom of the lowest video is within `1.5 * default_item_height` of the screen viewport.
- The app preloads the next video in the background so it attaches seamlessly the moment it scrolls into range.

### 3. Proximity-Based Audio Crossfading

Volume is modulated by the player's distance from the vertical center of the window:

```mermaid
graph TD
    Distance["Calculate Distance from Screen Center (d)"] --> Norm["Normalized Proximity: (1.0 - d / half_height)"]
    Norm --> Cosine["Apply smooth cosine curve"]
    Cosine --> Master["Scale by Master Volume"]
    Master --> Volume["Set Player Volume"]
```

---

## 📜 Subtitle Transcript Engine (`subtitles.rs`)

The subtitle subsystem extracts and parses timed dialogue lines:

1. **Sidecar File Parsing**: Checks for adjacent subtitle files (`.srt`, `.vtt`, `.ass`, `.ssa`).
2. **Container Stream Extraction**: Uses an asynchronous background `ffmpeg` process to extract embedded subtitle tracks without freezing playback.
3. **Timestamp Normalization**: Parses various timestamp conventions (`00:01:23,450`, `00:01:23.450`, `0:01:23.45`) into floating-point seconds ([`SubtitleCue`]).
4. **Interactive Seeking**: Clicking any cue in the transcript drawer dispatches a seek command to that cue's `start_secs`.
5. **Resource Limits & Sandboxing**: External sidecar reads are strictly bounded to 10 MB (`MAX_SUBTITLE_FILE_BYTES`) and parsed cues are capped at 50,000 (`MAX_SUBTITLE_CUES`) to prevent memory exhaustion and UI lockups. FFmpeg extractions enforce `-protocol_whitelist file,crypto` and validate stream mapping arguments.
