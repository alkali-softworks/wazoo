# <img src="public/icon.png" width="48" align="center" /> Wazoo

**Wazoo** is an ambient media engine designed for those who want to experience their local video collection without the burden of choice. Built as a "moving mood-board" for artists, designers, and curators, Wazoo provides a non-stop, feed of visuals that flows continuously based on your library and optional search queries.

Forget the play button. Just open Wazoo and let your media collection become the atmosphere.

---

## 📦 Pre-Built Desktop Executables

For a quick and effortless setup without compiling from source, you can download the latest pre-compiled desktop versions directly from the [Nightly Releases Page](https://github.com/alkali-softworks/wazoo/releases/tag/nightly/):

*   **Windows (`.exe` Installer)**: [Wazoo-1.0.0.Setup.exe](https://github.com/alkali-softworks/wazoo/releases/download/nightly/Wazoo-1.0.0.Setup.exe)
*   **macOS (`.zip` App Bundle)**: [Wazoo-darwin-arm64-1.0.0.zip](https://github.com/alkali-softworks/wazoo/releases/download/nightly/Wazoo-darwin-arm64-1.0.0.zip) (Apple Silicon arm64)
*   **Linux (`.deb` Package)**: [wazoo-desktop_1.0.0_amd64.deb](https://github.com/alkali-softworks/wazoo/releases/download/nightly/wazoo-desktop_1.0.0_amd64.deb)
*   **Linux (`.rpm` Package)**: [wazoo-desktop-1.0.0-1.x86_64.rpm](https://github.com/alkali-softworks/wazoo/releases/download/nightly/wazoo-desktop-1.0.0-1.x86_64.rpm)
*   **NuGet Package**: [wazoo_desktop-1.0.0-full.nupkg](https://github.com/alkali-softworks/wazoo/releases/download/nightly/wazoo_desktop-1.0.0-full.nupkg)

---

## 🧠 The Philosophy: "Zero-Choice Viewing"

Wazoo was born from a simple problem: **digital fatigue.** We spend more time scrolling through thumbnails than actually watching our media. 

Wazoo flips the script. Instead of making you "pick," it creates a **continuous, non-stop feed** based on your entire collection or a specific theme. It's designed to be:
*   **A Moving Mood-Board:** Perfect for artists needing background inspiration.
*   **An Ambient Engine:** For those who want visuals running in the corner of their vision.
*   **Effortless Discovery:** Rediscover forgotten gems in your library without ever having to click "Open File."

---

## ✨ Key Features

### 🖼️ Ambient Orchestration
Wazoo isn't a traditional player; it's a visual environment. Run multiple videos simultaneously in **Grid**, **Row**, or **Column** layouts. It automatically handles the layout, creating a dynamic media dashboard that stays active in the background of your workspace.

### 🌊 The Infinity Stream
Experience your local library as a living river of content. The **Scroll Mode** provides a vertical feed that plays videos at random indices while scrolling at a constant rate. With smooth audio cross-fading based on visibility, it transforms your hard drive into a curated, non-stop stream of inspiration.

### ⚡ H.265 (HEVC) Transcoding
Don't let codec limitations slow you down. Chrome has trouble with HEVC, but Wazoo features a built-in **FFmpeg-powered transcoding engine** that live-converts H.265 content for seamless playback in the Electron environment.

### 🔍 Smart Library Management
* **Instant Search:** Find any video in your collection in milliseconds.
* **Media Scanning:** Automatically indexes your folders into a lightning-fast local SQLite database.
* **Persistent State:** Wazoo remembers where you left off, including your last search and window layout.

### 🌍 Global by Design
Wazoo is built for everyone, with full localization support for **16+ languages**:
*   **RTL Ready:** Native support for Arabic and Hebrew with automatic layout mirroring.
*   **Multi-Lingual:** Support for English, Spanish, French, German, Japanese, Chinese, Hindi, Russian, and many more.
*   **Glassmorphism UI:** A sleek, semi-transparent interface that feels alive in any language.
*   **Alt-Drag Navigation:** Quickly move the window anywhere on your screen with a simple Alt+Drag.

## 🚀 Local Development & Building from Source

If you want to run the engine in development mode or build a package locally:

### Prerequisites
* [Node.js](https://nodejs.org/) (v20 or higher recommended)

### Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/alkali-softworks/wazoo.git
   cd wazoo-desktop
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run the development environment:
   ```bash
   npm run dev
   ```
4. Build / Package locally (for your current operating system):
   ```bash
   npm run package
   ```

---

## 🤝 Contributing

Wazoo is built for creators and enthusiasts. If you'd like to contribute, feel free to fork the repo or submit a PR!

## 📄 License

This project is licensed under the MIT License - Do whatever you want with it (but give credit where credit is due).

---

<p align="center">Made with ❤️ by <a href="mailto:contact@alkalisoftworks.com">Alkali Softworks</a></p>
