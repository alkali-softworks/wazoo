# 🌊 Wazoo-RS

> **Ambient media engine for non-stop viewing — rewritten in native Rust.**

**Wazoo-RS** is a high-performance native port of [Wazoo](https://github.com/alkali-softworks/wazoo), inspired by [Madamiru](https://github.com/mtkennerly/madamiru). Built as a "moving mood-board" for artists, designers, and curators, Wazoo transforms your local video collection into a non-stop, ambient atmosphere.

Forget the play button. Open Wazoo and let your media collection become the environment.

---

## ⚡ Why Native Rust?

The original Wazoo was built on Electron. While flexible, running multiple simultaneous video players inside Chromium resulted in significant memory usage and required an active FFmpeg transcoding pipeline for H.265/HEVC content.

**Wazoo-RS** replaces Electron with a native Rust architecture:

| Capability | Electron Desktop (`wazoo-desktop`) | Native Rust (`wazoo-rs`) |
| :--- | :--- | :--- |
| **RAM Footprint (4 players)** | ~1.5 GB – 3 GB | **~150 MB – 350 MB** |
| **H.265 / HEVC 10-bit Playback** | CPU transcode via live FFmpeg | **Native GPU hardware decoding** |
| **UI Framework** | Vue 3 + Chromium DOM | **Iced 0.14** (`wgpu` accelerated) |
| **Media Subsystem** | HTML5 `<video>` | **GStreamer 1.0** via `iced_video_player` |
| **Database** | `better-sqlite3` + Drizzle | **`rusqlite`** (bundled) with indexed queries |
| **Startup Time** | ~2–4 seconds | **< 150 ms** |

---

## ✨ Features

### 🖼️ Ambient Orchestration (Multi-Player Layouts)
Run 1 to 12 video players simultaneously in **Grid**, **Row**, or **Column** configurations. The responsive layout engine adapts automatically as players are added or removed.

### 🌊 The Infinity Stream ("Scroll Mode")
Experience your media library as a continuous vertical river of content:
- Constant, smooth vertical scroll at configurable `scroll_speed`.
- **Virtual Viewport**: Automatically despawns players that scroll off the top and spawns new players with random selections at the bottom edge.
- **Audio Cross-Fading**: Video audio smoothly cross-fades based on the percentage of the player visible on screen.

### ⚡ Flip Mode
Rapid ambient variety. Staggered background timers automatically shuffle and seek active video players to random timestamps, keeping visuals fresh without manual intervention.

### 🐕 Unstuck Watchdog
Ambient players should never freeze. Wazoo-RS monitors playback position progress and automatically cycles to the next video if a stream encounters an issue or stalls.

### 🔍 Instant Library Indexing & Search
- Recursive directory scanning with title cleaning (removes resolution tags, brackets, and clutter).
- Sniffs codec, duration, dimensions, and subtitle tracks.
- Instant search supporting comma-separated terms, `not <term>` exclusions, and folder scoping.

### 🪟 Frameless Transparent Windowing
- Transparent, borderless window designed to blend seamlessly into your desktop.
- **Alt + Drag** navigation to position the window anywhere on screen.
- Global mute / unmute toggle and volume management.

---

## 🏗️ Architecture & Crates

The project is organized as a Cargo workspace with four modular crates:

```
wazoo-rs/
├── crates/
│   ├── wazoo-core/       # SQLite database, schema, models, & settings persistence
│   ├── wazoo-scanner/    # Multithreaded media discovery, name cleaning, & metadata probing
│   ├── wazoo-media/      # GStreamer pipeline wrappers, VideoHandle, & ScrollEngine
│   └── wazoo-app/        # Iced 0.14 native desktop GUI (layouts, HUD, modals)
└── Cargo.toml
```

---

## 🚀 Getting Started

### Prerequisites

#### Linux (Ubuntu/Debian)
Install GStreamer 1.0 runtime and development packages:
```bash
sudo apt-get install \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav
```
*Note: A local sysroot configuration is bundled in `.cargo/config.toml` for zero-install environments.*

#### Windows & macOS
GStreamer binaries can be downloaded from the [GStreamer official website](https://gstreamer.freedesktop.org/download/).

---

### Building & Running

1. **Clone the repository:**
   ```bash
   git clone https://github.com/alkali-softworks/wazoo-rs.git
   cd wazoo-rs
   ```

2. **Run all unit & integration tests:**
   ```bash
   cargo test
   ```

3. **Launch the application:**
   ```bash
   cargo run --bin wazoo
   ```

4. **Build an optimized release executable:**
   ```bash
   cargo build --release --bin wazoo
   ```

---

## ⌨️ Controls & Shortcuts

| Shortcut / Key | Function |
| :--- | :--- |
| `F` / `J` / `/` | **Find** — Open Instant Search modal |
| `S` | Toggle **Shuffle** vs. Sequential playback mode |
| `1` | Set player count to **1** |
| `2` | Set player count to **2** |
| `3` | Set player count to **3** |
| `4` | Set player count to **4** |
| `5` | Toggle **Scroll Mode** (The Infinity Stream) |
| `6` | Toggle **Flip Mode** (staggered auto-shuffle) |
| `Space` | Play / Pause focused player |
| `ArrowUp` | Play next video on focused player |
| `ArrowDown` | Play previous video on focused player |
| `ArrowLeft` / `Right` | Seek backward / forward 5 seconds |
| `,` / `.` | Step frame backward / forward |
| `L` | Cycle Layout (`Grid` ➔ `Row` ➔ `Column`) |
| `N` | Add new player (up to 12) |
| `X` | Remove focused player |
| `Tab` | Cycle focused player |
| `M` | Toggle Mute (Global in scroll mode) |
| `[` / `]` | Decrease / Increase volume |
| `-` / `+` | Decrease / Increase scroll speed |
| `C` | Toggle Subtitles |
| `T` | Toggle Title / Info overlay |
| `H` | Toggle Controls HUD |
| `Esc` | Close modal / open Menu |
| `Alt + Drag` | Move borderless window |

---

## 📄 License

This project is licensed under the **MIT License** — see the LICENSE file for details.

Made with ❤️ by [Alkali Softworks](https://alkalisoftworks.com/).
