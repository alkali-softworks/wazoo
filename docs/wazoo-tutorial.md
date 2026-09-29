# `wazoo-tutorial` Subsystem Architecture

The [`wazoo-tutorial`] crate is an interactive educational desktop application and learning sandbox included within the Wazoo workspace.

---

## 🎯 Purpose & Goals

Building high-performance desktop applications with Rust, [Iced](https://github.com/iced-rs/iced), `tokio`, and `rusqlite` requires understanding several concepts:
- The Elm Architecture (Model-View-Update).
- Unidirectional data flow and immutable view rendering.
- Asynchronous task management (`iced::Task`) without blocking the UI thread.
- Custom widget styling and container theming.
- Thread-safe SQLite database interactions.

`wazoo-tutorial` provides a self-contained, live application where developers can explore, modify, and test these patterns in isolation.

---

## 📁 Module Organization

```
crates/wazoo-tutorial/src/
├── app.rs        # TutorialApp state machine, tab routing, and master layout
├── db.rs         # Standalone SQLite tutorial database helper
├── lib.rs        # Library entry point
├── main.rs       # Executable entry point with comprehensive step-by-step commentary
├── message.rs    # Event messages partitioned by tutorial tab
├── style.rs      # Design tokens, color schemes, and widget style helpers
└── tabs/         # Individual interactive learning modules
    ├── async_scanner_tab.rs # Async Tokio tasks, rfd native file dialogs, progress bars
    ├── counter_tab.rs       # The Elm Architecture basics (State, Message, View)
    ├── database_tab.rs      # SQLite CRUD operations and table rendering
    ├── guide_tab.rs         # Conceptual overview of Iced and Rust GUI architecture
    ├── mod.rs               # Tabs module declarations
    └── theming_tab.rs       # Custom styling, dark mode palettes, and container themes
```

---

## 🕹️ Interactive Learning Tabs

### 1. Conceptual Guide (`guide_tab.rs`)
Explains how Iced initializes native windows via `winit`, acquires GPU contexts via `wgpu`, manages an async event loop via `tokio`, and drives the reactive frame loop.

### 2. The Elm Architecture Basics (`counter_tab.rs`)
Demonstrates core TEA principles:
- State definitions (`struct CounterState`).
- Intent definitions (`enum CounterMessage`).
- Pure view functions that return declarative widget trees.

### 3. Theming & Styling (`theming_tab.rs`)
Demonstrates how to build rich, modern dark-mode interfaces in Iced without relying on CSS or web runtimes. Covers custom borders, glassmorphic container backgrounds, and button interaction states (hover, pressed).

### 4. SQLite Persistence (`database_tab.rs`)
Demonstrates thread-safe database operations:
- In-memory and on-disk SQLite databases using `rusqlite`.
- Binding parameters safely to prevent SQL injection.
- Dynamic table rendering and reactive record insertion.

### 5. Asynchronous Background Tasks (`async_scanner_tab.rs`)
Illustrates how Wazoo runs background media scans:
- Opening native OS file/folder pickers asynchronously via `rfd`.
- Spawning background `tokio` worker threads.
- Streaming real-time progress events back to the UI via `iced::Task::run` channels without stalling the 60 FPS presentation loop.

---

## 🚀 Running the Tutorial

To run the interactive tutorial sandbox:

```bash
cargo run -p wazoo-tutorial
```

Or with debug logging enabled:

```bash
RUST_LOG=info cargo run -p wazoo-tutorial
```
