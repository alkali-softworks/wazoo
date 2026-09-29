# <img src="crates/wazoo-app/resources/icon.png" width="48" align="center" /> Wazoo Video Player

> **Ambient multi-player for non-stop viewing.**

**Wazoo** is a video player and "moving mood-board" made for artists and curators who want a continuous stream of visual reference and inspiration from their local video collection. It provides a continuous **multi-player** view based on your library and optional search query. 

Wazoo is an idea in development since 2020 - originally as a JS app, then Electron, and now finally a Rust app, ported with all the best features from prior versions.

---

## 📦 Desktop Builds

For a quick setup without compiling from source, you can download the latest pre-compiled build

*   **Windows (`.exe` Standalone)**: [wazoo-windows-x86_64.exe](https://github.com/alkali-softworks/wazoo/releases/download/nightly/wazoo-windows-x86_64.exe) (portable single file with embedded libmpv)
*   **Windows (`.zip` Archive)**: [wazoo-windows-x86_64.zip](https://github.com/alkali-softworks/wazoo/releases/download/nightly/wazoo-windows-x86_64.zip)
*   **macOS (`.zip` - Apple Silicon)**: [wazoo-macos-arm64.zip](https://github.com/alkali-softworks/wazoo/releases/download/nightly/wazoo-macos-arm64.zip) (Apple Silicon M1/M2/M3/M4)
*   **macOS (`.zip` - Intel)**: [wazoo-macos-x86_64.zip](https://github.com/alkali-softworks/wazoo/releases/download/nightly/wazoo-macos-x86_64.zip) (Intel x86_64)
*   **Linux (`.AppImage` Standalone)**: [wazoo-linux-x86_64.AppImage](https://github.com/alkali-softworks/wazoo/releases/download/nightly/wazoo-linux-x86_64.AppImage) (Portable single executable)
*   **Linux (`.tar.gz` Archive)**: [wazoo-linux-x86_64.tar.gz](https://github.com/alkali-softworks/wazoo/releases/download/nightly/wazoo-linux-x86_64.tar.gz)

---

## Features

### 🧠 High-Performance Architecture
- Built in native **Rust** and **Iced** with **`libmpv`** (plays virtually any video, just like VLC) 
- Fast **SQLite** indexing: Query tens of thousands of video files instantly.
- Detailed architecture documented in [**docs/**](docs/README.md).

### 🖼️ Multi-Tile Layouts
- **1 to 12 Video Players**: Scale from a single minimalist player to an ambient video wall.
- **Flexible Layouts**: Cycle between **Grid**, **Row**, and **Column** arrangements (<kbd>L</kbd>).
- **Independent Audio & Controls**: Per-tile volume control, play mode, and language select.

### 🌊 Vertical Feed / Scroll Mode
View your library as a continuous vertical feed (<kbd>5</kbd>):
- **Vertical scroll speed** is configurable via <kbd>+</kbd> / <kbd>-</kbd>.
- **Audio Cross-Fading**: Volume adjusts with scroll position, proportional to video visibility.

### ⚡ Flip Mode & Shuffle History
Keeps background visuals fresh by rotating videos on staggered timers (<kbd>6</kbd>):
- **Jump Back to Missed Videos**: If you catch an interesting frame right as it flips away, press **Previous** (<kbd>↓</kbd>) to return to that exact video and timestamp.
- **Scrub Back & Forth**: Press **Next** (<kbd>↑</kbd>) to step forward through your recently viewed stack before generating new random picks.
- **Toggle Modes**: Press <kbd>S</kbd> to switch between **Shuffle** and **Sequential** playback (clears navigation history so you can step sequentially through files).

### 🔍 Instant Library Search
- **Directory Scanner**: Recursively indexes media folders while cleaning up messy filenames in the UI.
- **Dial in your media pool**:  Type partial names, separate multiple terms with commas, and exclude terms with `not ` or `!` (e.g. `anime, not bebop`).

### 📁 File Browser Drawer
- **Collapsible Directory Tree**: Press <kbd>F</kbd> to browse folders with file counts.
- **Direct Tile Loading**: Click any file to load it into the active player.

### 📜 Interactive Subtitle Transcript Drawer
- **Live Transcript**: View the active video's dialogue in a side drawer (<kbd>V</kbd> or <kbd>[TX]</kbd>).
- **Click to Seek**: Click any subtitle line to jump directly to that timestamp - great for studying dialogue or language learning.
- **Searchable**: Filter lines to locate specific quotes or scenes.

### 🔖 Bookmarks & Scene Memory
- **Save Scenes**: Press <kbd>+</kbd> or <kbd>=</kbd> to bookmark the current video, timestamp, and search query.
- **Quick Recall**: Open the bookmarks drawer (<kbd>B</kbd>) to return to any saved scene in one click.
- **Auto Save**: Wazoo automatically restores last open search + videos on launch.

### 🎞️ Frame-Stepping & Mark-In / Mark-Out Looping
- **Frame-by-Frame Stepping**: Press <kbd>,</kbd> (`<`) or <kbd>.</kbd> (`>`) to step backward or forward by one video frame.
- **Set Mark In**: Press <kbd>I</kbd> to set a loop start point (loops to the end of the video if no Out point is set).
- **Set Mark Out**: Press <kbd>O</kbd> to set a loop end point (loops from the beginning if no In point is set).
- **Visual Feedback**: The active loop range displays as an interactive badge in the player control bar.

### 🪟 Frameless Window & HUD
- Borderless window with a sliding titlebar, quick drop-down menu, and HUD controls.
- **Alt + Drag** to reposition the window anywhere on screen.
- Configurable **Window Opacity** (in System Settings).

### 📌 Always on Top (Click-Through Overlay)
- Pinning engages click-through mouse passthrough when Wazoo is unfocused, allowing you to interact with applications underneath.
- Pair Pin Mode with **Window Opacity** (in System Settings) to turn Wazoo into a semi-transparent, floating reference board. 
- **Escape Hatch**: Focusing Wazoo on your OS taskbar instantly disengages mouse passthrough and enables controls.

### 🌐 Internationalization & Localization
- **16 Supported Languages**: Complete UI translations for English (`en`), Spanish (`es`), French (`fr`), German (`de`), Italian (`it`), Portuguese (`pt`), Russian (`ru`), Simplified Chinese (`zh`), Japanese (`ja`), Korean (`ko`), Thai (`th`), Hebrew (`he`), Arabic (`ar`), Hindi (`hi`), Bengali (`bn`), and Indonesian (`id`).

---

## 🚀 Getting Started

### 1. Launching Wazoo

Download a pre-compiled standalone binary or archive from [Desktop Builds](#-desktop-builds).

On first launch, Wazoo displays the welcome screen where you can click **Add Folder** to select your media folders. Media files are automatically scanned and indexed in the background into an ultra-fast local SQLite database.

### 2. Command-Line Launch & Direct Playback

You can also launch Wazoo directly from your terminal with folders, video files, or search queries:

```bash
# Open Wazoo pointing to a specific media folder
wazoo /path/to/videos

# Play an individual video directly
wazoo movie.mkv

# Launch pre-filtered to a specific search query
wazoo "bebop"
```

### 3. Essential Shortcuts

- <kbd>F1</kbd> — Help / Keybinds Reference
- <kbd>Space</kbd> — Play / Pause
- <kbd>F</kbd> – Toggle **File Drawer**
- <kbd>J</kbd> or <kbd>/</kbd> — Open the **Search** modal
- <kbd>S</kbd> — Toggle **Shuffle vs Sequential** playback

---

## 🛠️ Building from Source

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

### Compilation & Packaging

1. **Clone the repository:**
   ```bash
   git clone https://github.com/alkali-softworks/wazoo.git
   cd wazoo
   ```

2. **Build a self-contained release executable:**
   ```bash
   cargo build --release
   ```
   - **Windows:** Outputs `target/release/wazoo.exe` with all runtime dependencies packed into the single `.exe` file.
   - **Linux / macOS:** Outputs `target/release/wazoo` with portable dynamic linking.

3. **Package a Linux Single-Executable AppImage (Optional):**
   ```bash
   ./scripts/build-appimage.sh
   ```
   Generates `dist/wazoo-x86_64.AppImage`.

---

## ⌨️ Controls & Shortcuts

| Shortcut / Key | Action |
| :--- | :--- |
| <kbd>N</kbd> | Add player (up to 12) |
| <kbd>X</kbd> | Remove focused player |
| <kbd>L</kbd> | Cycle Layout (**Grid** ➔ **Row** ➔ **Column**) |
| <kbd>Tab</kbd> | Focus next player |
| <kbd>J</kbd> / <kbd>/</kbd> | Open **Find / Search** modal |
| <kbd>H</kbd> / <kbd>F</kbd> | Toggle **File Browser** drawer |
| <kbd>S</kbd> | Toggle **Shuffle** vs. **Sequential** playback |
| <kbd>P</kbd> | Toggle **Pin Mode** (Always on Top) |
| <kbd>V</kbd> | Toggle **Interactive Subtitle Transcript** drawer |
| <kbd>B</kbd> | Toggle **Bookmarks** modal |
| <kbd>Y</kbd> | Toggle **Play History** drawer |
| <kbd>C</kbd> | Toggle Subtitles on/off |
| <kbd>M</kbd> | Toggle **Mute** (unmutes when volume changes) |
| <kbd>[</kbd> / <kbd>]</kbd> | Adjust Volume down / up |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Next / Previous video in focused player |
| <kbd>←</kbd> / <kbd>→</kbd> | Seek backward / forward 5 seconds |
| <kbd>1</kbd> – <kbd>4</kbd> | Set active player count (1 to 4) |
| <kbd>5</kbd> | Toggle **Scroll Mode** (vertical feed) |
| <kbd>6</kbd> | Toggle **Flip Mode** (staggered auto-shuffle) |
| <kbd>+</kbd> / <kbd>=</kbd> | Add Bookmark (or increase Scroll Speed in Scroll Mode) |
| <kbd>-</kbd> | Remove Bookmark (or decrease Scroll Speed in Scroll Mode) |
| <kbd>I</kbd> / <kbd>O</kbd> — Set **Mark In** / **Mark Out** (A-B loop) |
| <kbd>Alt+I</kbd> / <kbd>Alt+O</kbd> — Clear **Mark In** / **Mark Out** |
| <kbd>R</kbd> | **Random Seek** on focused player |
| <kbd>T</kbd> | Show video titles (all players) |
| <kbd>,</kbd> / <kbd>.</kbd> | Step backward / forward one frame |

---

## ⚙️ Configuration & Environment Variables

### Configuration & Database Files (`settings.json`, `wazoo.db`)

Wazoo automatically persists user preferences, window state, and media library indexing together in the platform configuration directory:

- **Linux:** `~/.config/wazoo/` (`settings.json`, `wazoo.db`)
- **Windows:** `%APPDATA%\alkalisoftworks\wazoo\config\` (`settings.json`, `wazoo.db`)
- **macOS:** `~/Library/Application Support/com.alkalisoftworks.wazoo/` (`settings.json`, `wazoo.db`)

Key properties configurable in `settings.json`:

```json
{
  "flip_interval_secs": 45,
  "buffer_duration_secs": 10,
  "buffer_size_mb": 32,
  "window_opacity": 1.0,
  "language": "en",
  "preferred_audio_language": "Japanese",
  "keybinds": {}
}
```
- **`buffer_duration_secs`**: Demuxer readahead buffer duration in seconds (range: 2–300s, default: `10`).
- **`buffer_size_mb`**: Maximum demuxer cache size in megabytes (range: 16–4096 MB, default: `32`).

### Environment Variables

| Variable | Description | Default |
| :--- | :--- | :--- |
| `WAZOO_HWDEC` | Overrides the `libmpv` hardware video decoding profile (e.g. `auto-copy`, `vaapi`, `nvdec`, `no`). Software decoding is used by default (`no`) to prevent thread deadlocks and driver issues when windows are occluded or behind other applications. | `no` |
| `WGPU_POWER_PREF` | Selects GPU power profile for the Iced/WGPU renderer (`low-power` or `high-performance`). Automatically defaults to `low-power` on Linux dual-GPU hybrid laptops when unset. | `low-power` / system default |
---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

Made with ❤️ by [Alkali Softworks](https://alkalisoftworks.com/).
