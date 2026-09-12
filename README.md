# 🌊 Wazoo

> **Ambient media engine for non-stop viewing — built in native Rust.**

**Wazoo** is a high-performance ambient media engine and "moving mood-board" designed for artists, designers, collectors, and curators. It transforms your local video collection into a vibrant, non-stop ambient atmosphere.

Forget the play button. Open Wazoo and let your media collection become the room.

---

## ⚡ High-Performance Architecture

Built from the ground up in native Rust with hardware-accelerated rendering:

- **Ultra-Lightweight Footprint**: Consumes only ~150 MB – 350 MB of RAM across multiple simultaneous high-definition streams.
- **Native GPU Hardware Decoding**: Full hardware-accelerated playback for H.264, H.265/HEVC 10-bit, AV1, and VP9 through `libmpv`.
- **`wgpu`-Accelerated Interface**: Rendered with **Iced** using GPU rendering pipelines for 60fps animations and glassmorphic overlays.
- **Blazing Fast Startup**: Instant sub-150ms launch time with automated window manager focus acquisition.
- **Embedded SQLite Index**: Powered by `rusqlite` for lightning-fast queries across tens of thousands of video files.

---

## ✨ Features


### 🖼️ Ambient Orchestration & Multi-Tile Layouts
- **1 to 12 Video Players**: Scale from a single minimalist player to an ambient video wall.
- **Responsive Layout Engine**: Effortlessly cycles between **Grid**, **Row**, and **Column** arrangements (<kbd>L</kbd>).
- **Independent Audio & Seeking**: Per-player volume control, seek sliders, auto-unmute on volume adjustment, and quick random seeking (<kbd>R</kbd>).

### 🌊 The Infinity Stream ("Scroll Mode")
Experience your media library as a continuous vertical river of content (<kbd>5</kbd>):
- Constant, smooth vertical scroll at configurable `scroll_speed` (<kbd>+</kbd> / <kbd>-</kbd>).
- **Virtual Viewport**: Automatically despawns players that scroll off the top edge and seamlessly spawns new candidate videos at the bottom.
- **Proportional Audio Cross-Fading**: Sound smoothly fades in and out based on the visible percentage of each video on screen.

### ⚡ Flip Mode
Rapid ambient variety (<kbd>6</kbd>). Staggered background timers automatically shuffle and seek active video tiles to random timestamps, keeping visuals fresh without manual intervention.

### 🔍 Instant Library Indexing & Search
- **Smart Directory Scanner**: Recursively indexes media folders while stripping clutter from video titles.
- **Advanced Query Parser**: Supports instant filtering, folder scoping, and comma-separated search terms.
- **Adaptive Player Reconciliation**: Executing a new search dynamically updates active tiles to ensure all players stream matching content without duplicates.

### 🪟 Frameless Glassmorphic Interface
- Borderless window with sliding titlebar, quick drop-down menu, and glassmorphic HUD controls.
- **Alt + Drag** navigation to position the window anywhere on screen.
- Configurable window opacity for semi-transparent ambient desktop backgrounds.

### 📜 Interactive Subtitle Transcript Drawer
- **Live Dialogue Transcript**: View a full transcript of the active video dialogue in a sleek, sidecar slide-out drawer (<kbd>V</kbd> or <kbd>[TX]</kbd> button).
- **Click to Seek**: Click any subtitle line to jump video playback directly to that timestamp instantly.
- **Real-Time Active Line Highlighting**: The currently spoken dialogue cue lights up with an emerald highlight and timestamp badge in real time as the video plays.
- **Dialogue Search Filter**: Filter thousands of dialogue lines in real-time to find exact quotes and scenes.
- **Universal Subtitle Extraction**: Automatically parses external sidecar files (`.srt`, `.vtt`, `.ass`, `.ssa`) or demuxes embedded subtitle tracks in milliseconds on a background thread.

### 🔖 Bookmarks & Scene Memory
- **Instant Bookmarking**: Press <kbd>+</kbd> or <kbd>=</kbd> to save a bookmark of the current video, exact timestamp, search query, and playback mode.
- **Scene Restoration**: Open the bookmarks drawer (<kbd>B</kbd>) and jump back to any bookmarked scene with a single click.
- **Multi-Player Reconciliation**: Restoring a bookmark restores your global search query and automatically swaps active player tiles to matching media.

### 📁 File Browser Drawer
- **Collapsible Folder Tree**: Press <kbd>H</kbd> to slide out the library directory tree, organized cleanly with file counts.
- **Direct Tile Loading**: Click any file in the drawer to immediately load it into the currently focused player.

### 💻 Command-Line Launch & Search
- **Instant CLI Boot**: Launch Wazoo directly with a search query from your terminal:
  ```bash
  wazoo "ambient"
  ```
- **Console Focus Acquisition**: Automatically claims active OS window focus upon startup so keyboard hotkeys work immediately without clicking.

### 🐕 Unstuck Playback Watchdog
Ambient displays must never stall. Wazoo constantly monitors playback position progress and automatically cycles to the next video if a network stream or corrupted file encounters a stall.

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
`mpv-2.dll` can be obtained from the official mpv builds or libmpv development distributions.

---

### Building & Running

1. **Clone the repository:**
   ```bash
   git clone https://github.com/alkali-softworks/wazoo-rs.git
   cd wazoo-rs
   ```

2. **Run the test suite:**
   ```bash
   cargo test --workspace -- --test-threads=1
   ```

3. **Build an optimized release binary:**
   ```bash
   cargo build --release
   ```

4. **Launch Wazoo:**
   ```bash
   ./target/release/wazoo
   ```

---

## ⌨️ Controls & Shortcuts

| Shortcut / Key | Action |
| :--- | :--- |
| <kbd>V</kbd> | Toggle **Interactive Subtitle Transcript** drawer |
| <kbd>B</kbd> | Toggle **Bookmarks** modal |
| <kbd>+</kbd> / <kbd>=</kbd> | Add Bookmark (or increase Scroll Speed in Scroll Mode) |
| <kbd>-</kbd> | Remove Bookmark (or decrease Scroll Speed in Scroll Mode) |
| <kbd>R</kbd> | **Random Seek** on focused player |
| <kbd>T</kbd> | Show video titles (all players) |
| <kbd>H</kbd> | Toggle **File Browser** drawer |
| <kbd>C</kbd> | Toggle Subtitles on/off |
| <kbd>J</kbd> / <kbd>F</kbd> / <kbd>/</kbd> | Open **Find / Search** modal |
| <kbd>S</kbd> | Toggle **Shuffle** vs. Sequential playback |
| <kbd>M</kbd> | Toggle **Mute** (unmutes automatically when volume changes) |
| <kbd>[</kbd> / <kbd>]</kbd> | Adjust Volume down / up |
| <kbd>Space</kbd> | Play / Pause focused player |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Next / Previous video in focused player |
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

## 📄 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

Made with ❤️ by [Alkali Softworks](https://alkalisoftworks.com/).
