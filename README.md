# 🌊 Wazoo

> **Ambient media engine for non-stop viewing - built in native Rust.**

**Wazoo** is an ambient video player and "moving mood-board" built for artists and curators who want a continuous stream of visual reference and inspiration from their local video collection. It provides a non-stop feed of videos based on your library and optional search query. 

Wazoo is an idea in development since 2020 - originally as a JS app, then Electron, and now finally a native Rust app, ported with the culmination of all the best features from prior versions.

---

## 🧠 Built for Background Reference

Most video players assume you're sitting down for a movie or managing a queue. Wazoo is built for a different workflow:

- **Hands-off ambient flow**: Point Wazoo at your video folders and let it run. No playlists to manage, no file dialogs to click through.
- **Fast search & filtering**: Use text queries and negative terms to instantly narrow down files and folders across your entire collection.

---

## ⚡ High-Performance Architecture

Built in native **Rust** and **Iced** with hardware-accelerated **`libmpv`**, so it can run all day in the background without hogging system resources:

- **Lightweight resource usage**: Consumes **~70 MB - 150 MB** of RAM during playback, leaving your CPU and GPU free for Blender, Photoshop, or your DAW.
- **Pure native stack**: Zero Electron or web-engine overhead, with instant startup and no garbage collection stutter.
- **Hardware-accelerated decoding**: Smooth playback for H.264, HEVC 10-bit, AV1, and VP9 through `libmpv`.
- **Fast local indexing**: SQLite (`rusqlite`) indexes tens of thousands of video files for instant search and shuffle queries.

---

## ✨ Features


### 🖼️ Multi-Tile Layouts
- **1 to 12 Video Players**: Scale from a single minimalist player to an ambient video wall.
- **Flexible Layouts**: Cycle between **Grid**, **Row**, and **Column** arrangements (<kbd>L</kbd>).
- **Independent Audio & Controls**: Per-tile volume control, seek sliders, auto-unmute on volume adjustment, and quick random seek (<kbd>R</kbd>).

### 🔍 Instant Library Search
- **Directory Scanner**: Recursively indexes media folders while cleaning up messy filenames in the UI.
- **Query Parser & Negative Search**: Dial in your media pool quickly. Type partial names, separate multiple terms with commas, and exclude terms with `not ` or `!` (e.g. `anime, not bebop`).

### 🌊 The Infinity Stream ("Scroll Mode")
View your library as a continuous vertical feed (<kbd>5</kbd>):
- Smooth vertical scroll at configurable `scroll_speed` (<kbd>+</kbd> / <kbd>-</kbd>).
- **Virtual Viewport**: Automatically unloads players that scroll out of view and loads new candidate videos at the bottom.
- **Proportional Audio Fading**: Audio fades in and out based on how much of the video tile is visible on screen.

### ⚡ Flip Mode & Shuffle History
Keeps background visuals fresh by rotating videos on staggered timers (<kbd>6</kbd>):
- **Jump Back to Missed Videos**: If you catch an interesting frame right as it flips away, press **Previous** (<kbd>↓</kbd>) to return to that exact video and timestamp.
- **Scrub Back & Forth**: Press **Next** (<kbd>↑</kbd>) to step forward through your recently viewed stack before generating new random picks.
- **Toggle Modes**: Press <kbd>S</kbd> to switch between **Shuffle** and **Sequential** playback (clears navigation history so you can step sequentially through files).

### 📜 Interactive Subtitle Transcript Drawer
- **Live Transcript**: View the active video's dialogue in a side drawer (<kbd>V</kbd> or <kbd>[TX]</kbd>).
- **Click to Seek**: Click any subtitle line to jump directly to that timestamp - great for studying dialogue or language learning.
- **Active Line Highlighting**: Follows spoken dialogue in real time.
- **Searchable**: Filter lines to locate specific quotes or scenes.

### 🔖 Bookmarks & Scene Memory
- **Save Scenes**: Press <kbd>+</kbd> or <kbd>=</kbd> to bookmark the current video, timestamp, search query, and playback mode.
- **Quick Recall**: Open the bookmarks drawer (<kbd>B</kbd>) to return to any saved scene in one click.

### 🪟 Frameless Window & HUD
- Borderless window with a sliding titlebar, quick drop-down menu, and HUD controls.
- **Alt + Drag** to reposition the window anywhere on screen.
- Configurable window opacity to keep visual reference floating subtly over other tools.

### 📁 File Browser Drawer
- **Collapsible Directory Tree**: Press <kbd>H</kbd> to browse folders with file counts.
- **Direct Tile Loading**: Click any file to load it into the active player.

### 🕒 Play History Drawer
- **Session History**: Logs every video played during your session (<kbd>Y</kbd> or right-click menu).
- **Click to Replay**: Click any entry in history to reload it in the active player.
- **Search & Filter**: Filter played history by title, directory, or filename.

### 💻 Command-Line Launch & Search
- Launch Wazoo with a search query directly from your terminal:
  ```bash
  wazoo "bebop"
  ```

### 🐕 Playback Watchdog
Monitors playback progress and automatically advances to the next video if a file stalls or encounters an issue, keeping your background stream running uninterrupted.

---

## 🏗️ Architecture

Wazoo is structured as a modular Cargo workspace:

```
wazoo-rs/
├── crates/
│   ├── wazoo-core/       # SQLite database, schema, models, & settings persistence
│   ├── wazoo-scanner/    # Multithreaded media discovery, name cleaning, & metadata probing
│   ├── wazoo-media/      # libmpv rendering pipeline, subtitle parser, VideoHandle, & ScrollEngine
│   └── wazoo-app/        # Iced native GUI (layouts, HUD overlays, modals, drawers)
└── Cargo.toml
```

---

## 🚀 Getting Started

### Prerequisites

#### Linux (Ubuntu/Debian)
Install `libmpv` runtime and development headers:
```bash
sudo apt-get install libmpv-dev libmpv2 ffmpeg
```

#### macOS
```bash
brew install mpv ffmpeg
```

#### Windows
No manual library setup or separate DLLs required! The build process automatically embeds and pre-compresses `libmpv` into a 100% self-contained, single-file executable (`wazoo.exe`) with official application icon and PE metadata embedded.

*(Optional)* For embedded subtitle extraction in the transcript drawer, install `ffmpeg`:
```powershell
winget install Gyan.FFmpeg
# or
scoop install ffmpeg
```

---

### Building & Packaging

1. **Clone the repository:**
   ```bash
   git clone https://github.com/alkali-softworks/wazoo-rs.git
   cd wazoo-rs
   ```

2. **Build a self-contained single executable:**
   ```bash
   cargo build --release
   ```
   - **Windows:** Outputs `target/release/wazoo.exe` with all runtime dependencies packed into the single `.exe` file.
   - **Linux:** Outputs `target/release/wazoo` with portable dynamic linking.

3. **Package a Linux Single-Executable AppImage (Optional):**
   ```bash
   ./scripts/build-appimage.sh
   ```
   Generates `dist/wazoo-x86_64.AppImage`.

---

## ⌨️ Controls & Shortcuts

| Shortcut / Key | Action |
| :--- | :--- |
| <kbd>V</kbd> | Toggle **Interactive Subtitle Transcript** drawer |
| <kbd>B</kbd> | Toggle **Bookmarks** modal |
| <kbd>Y</kbd> | Toggle **Play History** drawer (capped to 1,000 session videos) |
| <kbd>+</kbd> / <kbd>=</kbd> | Add Bookmark (or increase Scroll Speed in Scroll Mode) |
| <kbd>-</kbd> | Remove Bookmark (or decrease Scroll Speed in Scroll Mode) |
| <kbd>R</kbd> | **Random Seek** on focused player |
| <kbd>T</kbd> | Show video titles (all players) |
| <kbd>H</kbd> | Toggle **File Browser** drawer |
| <kbd>C</kbd> | Toggle Subtitles on/off |
| <kbd>J</kbd> / <kbd>F</kbd> / <kbd>/</kbd> | Open **Find / Search** modal |
| <kbd>S</kbd> | Toggle **Shuffle** vs. Sequential playback (resets navigation stack) |
| <kbd>M</kbd> | Toggle **Mute** (unmutes automatically when volume changes) |
| <kbd>[</kbd> / <kbd>]</kbd> | Adjust Volume down / up |
| <kbd>Space</kbd> | Play / Pause focused player |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Next / Previous video in focused player (navigates random history in shuffle mode) |
| <kbd>←</kbd> / <kbd>→</kbd> | Seek backward / forward 5 seconds |
| <kbd>,</kbd> / <kbd>.</kbd> | Step backward / forward one frame |
| <kbd>1</kbd> – <kbd>4</kbd> | Set active player count (1 to 4) |
| <kbd>5</kbd> | Toggle **The Infinity Stream** (Scroll Mode) |
| <kbd>6</kbd> | Toggle **Flip Mode** (staggered auto-shuffle) |
| <kbd>L</kbd> | Cycle Layout (**Grid** ➔ **Row** ➔ **Column**) |
| <kbd>N</kbd> | Add player (up to 12) |
| <kbd>X</kbd> | Remove focused player |
| <kbd>Tab</kbd> | Focus next player |
| <kbd>?</kbd> | Open **Keyboard Shortcuts** reference modal |
| <kbd>Esc</kbd> | Dismiss active drawer / modal, or open Quick Menu |
| <kbd>Alt + Drag</kbd> | Move borderless window |
| <kbd>Alt + X</kbd> | Quit application |

---

## ⚙️ Configuration & Environment Variables

### Environment Variables

| Variable | Description | Default |
| :--- | :--- | :--- |
| `WAZOO_FLIP_INTERVAL` | Overrides the Flip Mode rotation interval in seconds. Clamped between 1 and 3600 seconds. | `45` |
| `WAZOO_HWDEC` | Overrides the `libmpv` hardware video decoding profile (e.g. `auto-copy`, `vaapi`, `nvdec`, `no`). Software decoding is used by default (`no`) to prevent thread deadlocks and driver issues when windows are occluded or behind other applications. | `no` |
| `WGPU_POWER_PREF` | Selects GPU power profile for the Iced/WGPU renderer (`low-power` or `high-performance`). Automatically defaults to `low-power` on Linux dual-GPU hybrid laptops when unset. | `low-power` / system default |

### Configuration File (`settings.json`)

Wazoo automatically persists user preferences and window state in `settings.json`:

- **Linux:** `~/.config/wazoo-rs/settings.json`
- **Windows:** `%APPDATA%\alkalisoftworks\wazoo-rs\config\settings.json`
- **macOS:** `~/Library/Application Support/com.alkalisoftworks.wazoo-rs/settings.json`

Key properties configurable in `settings.json`:

```json
{
  "flip_interval_secs": 45,
  "buffer_duration_secs": 10,
  "buffer_size_mb": 64,
  "window_opacity": 1.0,
  "language": "en",
  "preferred_audio_language": "Japanese",
  "layout": "grid",
  "player_count": 1
}
```

- **`flip_interval_secs`**: Interval in seconds between random video switches in Flip Mode (range: 1–3600s, default: `45`). Can also be temporarily overridden with `WAZOO_FLIP_INTERVAL_SECS`.
- **`buffer_duration_secs`**: Demuxer readahead buffer duration in seconds (range: 2–300s, default: `10`).
- **`buffer_size_mb`**: Maximum demuxer cache size in megabytes (range: 16–4096 MB, default: `64`).
- **`window_opacity`**: Opacity of the main window (0.1 to 1.0, default: `1.0`).
- **`language`**: Interface language code (`en`, `es`, `ja`, `zh`, `de`, `fr`, etc.).
- **`preferred_audio_language`**: Preferred audio stream language for multi-track video playback.

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

Made with ❤️ by [Alkali Softworks](https://alkalisoftworks.com/).
