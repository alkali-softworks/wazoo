# ⚡ Wazoo Tutorial: Learn Rust & Iced by Building

Welcome to `wazoo-tutorial`! This is an interactive desktop tutorial application built with **Rust** and **Iced 0.14**, specifically designed to teach the core concepts of Rust and modern GUI development alongside the technologies used across `wazoo-rs`.

Every file in this crate is loaded with in-depth pedagogical comments explaining **why** Rust code is structured the way it is, how ownership and borrowing work in practice, and how to harness **The Elm Architecture (TEA)** to build lightning-fast, reactive native desktop apps.

---

## 🚀 Quick Start

From the root of `wazoo-rs` (`/home/klo/wazoo/wazoo-rs`):

```bash
# Run the interactive tutorial app
cargo run -p wazoo-tutorial

# Or with debug logging enabled to see events in the terminal:
RUST_LOG=info cargo run -p wazoo-tutorial

# Run the unit tests (verifies SQLite database persistence):
cargo test -p wazoo-tutorial
```

---

## 🗂️ What You Will Learn & File Tour

| File | What it Teaches | Key Technologies |
| :--- | :--- | :--- |
| [`src/main.rs`] | Application entry point, window settings, runtime loop | `iced::application`, `winit`, `wgpu` |
| [`src/message.rs`] | Events, algebraic data types, sum types, Rust enums | The Elm Architecture (TEA), Enums |
| [`src/app.rs`] | Central state, `update` logic, `Task<Message>`, subscriptions | Single Source of Truth, Borrow Checker |
| [`src/style.rs`] | Type-safe styling closures, cards, badges, buttons | `iced::Theme`, `Border`, `Shadow` |
| [`src/db.rs`] | Embedded SQLite persistence, CRUD, `Result` & `?` operator | `rusqlite`, Parameterized SQL |
| [`src/tabs/counter_tab.rs`] | Buttons, text input, slider, checkbox, pure views | `column!`, `row!`, `text_input`, `slider` |
| [`src/tabs/async_scanner_tab.rs`] | Non-blocking async background tasks, folder pickers | `rfd`, `walkdir`, `tokio`, `Task::perform` |
| [`src/tabs/database_tab.rs`] | Live UI for creating, listing, and deleting database records | SQLite CRUD, Dynamic lists |
| [`src/tabs/theming_tab.rs`] | Dynamic theme switching, palette color inspector | `pick_list`, `Theme::TokyoNight`, etc. |
| [`src/tabs/guide_tab.rs`] | Interactive in-app cheat sheet and reference guide | Built-in UI Reference |

---

## 🧩 Core Concepts Explained

### 1. The Elm Architecture (TEA)
Iced applications do not scatter mutable state across UI components (unlike React or typical OOP GUI toolkits). Instead, TEA enforces a unidirectional loop:

```
          ┌─────────────┐
          │    VIEW     │ ◄── Pure rendering function: view(&self) -> Element
          └──────┬──────┘
                 │ (user clicks button or moves slider)
                 ▼
          ┌─────────────┐
          │   MESSAGE   │ ◄── Rust enum variant (e.g. Message::IncrementCounter)
          └──────┬──────┘
                 │
                 ▼
          ┌─────────────┐
          │   UPDATE    │ ◄── Pure state transition: update(&mut self, message)
          └──────┬──────┘
                 │ (updates self.counter_value)
                 ▼
          (Iced re-renders VIEW)
```

### 2. Rust Ownership & Borrowing
- **`&self` (Immutable Borrow)**: In `view(&self)`, you only borrow read access to the app state. Rust guarantees the UI view cannot accidentally mutate data while rendering.
- **`&mut self` (Mutable Borrow)**: In `update(&mut self, message)`, the app has exclusive write access. The borrow checker ensures no data races can ever occur.
- **`String` vs `&str`**: `String` is an owned, heap-allocated string buffer. `&str` is a borrowed view into string characters. Notice how `text_input` borrows `&self.text_input_value`, avoiding needless memory allocations on every frame!

### 3. Background Async Tasks Without Freezing (`Task::perform`)
In media tools like `wazoo-rs`, scanning large video directories takes time. If done on the main thread, the window freezes.

In `src/tabs/async_scanner_tab.rs`, we run filesystem scanning asynchronously:
```rust
Task::perform(
    async move {
        // Runs on background threadpool without blocking GUI
        tokio::task::spawn_blocking(move || {
            // Walk filesystem with walkdir...
        }).await
    },
    Message::ScanFinished, // Dispatches message back into Iced when complete!
)
```

---

## 🎯 Hands-on Exercises to Try

Once you have run the app, try these fun challenges to level up your skills:

1. **Add a "Clear" button to the Text Input**:
   - Add a variant `ClearTextInput` to `enum Message` in [`src/message.rs`](file:///home/klo/wazoo/wazoo-rs/crates/wazoo-tutorial/src/message.rs).
   - In [`src/app.rs`](file:///home/klo/wazoo/wazoo-rs/crates/wazoo-tutorial/src/app.rs), handle `Message::ClearTextInput` by clearing `self.text_input_value.clear()`.
   - In [`src/tabs/counter_tab.rs`](file:///home/klo/wazoo/wazoo-rs/crates/wazoo-tutorial/src/tabs/counter_tab.rs), add a button with `.on_press(Message::ClearTextInput)`.

2. **Add a Note Filter/Search in the SQLite Tab**:
   - Add a search input field in [`src/tabs/database_tab.rs`](file:///home/klo/wazoo/wazoo-rs/crates/wazoo-tutorial/src/tabs/database_tab.rs).
   - Filter `self.cached_notes` by note title!

3. **Experiment with Custom Themes**:
   - In [`src/tabs/theming_tab.rs`](file:///home/klo/wazoo/wazoo-rs/crates/wazoo-tutorial/src/tabs/theming_tab.rs), add `Theme::CatppuccinMacchiato` or `Theme::Oxocarbon` to `AVAILABLE_THEMES` and observe how the entire UI styling transforms.
