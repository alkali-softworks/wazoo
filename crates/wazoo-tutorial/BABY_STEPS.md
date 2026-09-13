# 🦀 Rust for PHP & JS Survivors: Baby Steps Guide

*A no-nonsense handbook for understanding Rust & Iced without the academic jargon.*

---

## 📦 Chapter 1: Modules, Imports & Namespaces (`mod` vs `use`)

In Rust, **the compiler will completely ignore a file on your disk** unless you explicitly attach it to the module tree.

Here is the mental model in 3 simple rules:

---

### Rule 1: `mod` is NOT `import`. `mod` means "Compile this file!"

In JavaScript:
```js
// foo.js exists, and you just import it:
import { doSomething } from './foo.js';
```

In PHP:
```php
// With Composer PSR-4, you just:
use App\Models\User; // Autoloader finds User.php automatically
```

**In Rust, there is NO automatic file scanner/autoloader.**
The compiler starts at `main.rs` (the root of the tree) and stops. If you create a file called `src/message.rs`, the compiler literally doesn't even know it exists!

When you write in [`src/main.rs`]:
```rust
mod app;
mod db;
mod message;
mod style;
mod tabs;
```
You are telling the compiler:
> *"Hey rustc, go find `src/message.rs` (or `src/message/mod.rs`), parse it, compile it, and attach it to our program's module tree under the name `message`."*

You only declare `mod message;` **ONCE** in the root of your crate (usually `main.rs` or `lib.rs`).

---

### Rule 2: `use` is just a shortcut (like `import` or PHP's `use`)

Once a module is registered into the tree with `mod`, you use `use` to bring items into scope so you don't have to type full paths.

Without `use`:
```rust
// Annoying full path:
let msg = crate::message::Message::IncrementCounter;
```

With `use`:
```rust
use crate::message::Message;

// Clean shortcut:
let msg = Message::IncrementCounter;
```

---

### Rule 3: Namespaces & Paths (`crate::` vs `super::`)

Think of your crate as a file directory tree:

```text
crate (main.rs)
 ├── message (src/message.rs)
 ├── db (src/db.rs)
 └── tabs (src/tabs/mod.rs)
      ├── counter_tab (src/tabs/counter_tab.rs)
      └── database_tab (src/tabs/database_tab.rs)
```

Inside any file, you have three ways to reference things:

1. **`crate::`** — Starts at the very top (`src/main.rs`).
   ```rust
   // From anywhere in the project, start from the top:
   use crate::message::Message;
   use crate::db::TutorialDatabase;
   ```
   *(Think of this like an absolute path in bash: `/var/www/message`)*

2. **`super::`** — Moves up one parent module.
   ```rust
   // Inside src/tabs/counter_tab.rs, super:: refers to src/tabs/mod.rs
   use super::Message; 
   ```
   *(Think of this like `../` in bash or relative JS imports)*

3. **External Crates** (dependencies in `Cargo.toml`):
   ```rust
   // External libraries don't use `crate::`. They start with the library name:
   use iced::widget::button;
   use tokio::task;
   use rusqlite::Connection;
   ```

---

### Bonus Rule: Everything is `private` by default!

In PHP or JS classes/modules, things are often public or exported.
In Rust, **everything is private by default**:
- `struct Foo` can only be seen inside its own file.
- `pub struct Foo` can be seen by parent modules.
- `pub(crate) struct Foo` is visible to your whole project, but not to outside libraries.

That’s why in [`src/message.rs`]:
```rust
pub enum Message { ... }
// ^ Notice `pub`! Without `pub`, main.rs wouldn't be allowed to touch it.
```

### Summary Cheat Sheet

| You want to... | In JS/PHP | In Rust |
| :--- | :--- | :--- |
| Tell compiler a file exists | (Automatic) | `mod my_file;` (in `main.rs`) |
| Import a struct or function | `import { X }` / `use App\X` | `use crate::my_file::X;` |
| Import from a dependency | `import { X } from 'lib'` | `use lib::X;` |
| Make something visible | `export` / `public` | `pub` |

---

## 🔄 Chapter 2: What Happens at `.run()`? The Desktop Event Loop

In PHP, code runs from top to bottom and terminates:
```php
<?php
echo "Hello";
// Script dies here
```

In a desktop GUI application, calling `.run()` **hands the keys over to Iced and BLOCKS the main thread in an infinite loop**.

Here is the exact lifecycle of what happens inside `.run()`:

### Step 1: Memory Initialization (`TutorialApp::new`)
Iced calls your `new()` constructor once. Your application struct is allocated in RAM, default settings are populated, and SQLite is opened.

### Step 2: OS Window & GPU Pipeline Creation
Iced communicates with your OS window server (Wayland/X11 on Linux, Win32 on Windows) using the `winit` crate. It opens the window and asks `wgpu` to bind a WebGPU rendering pipeline directly to your GPU.

### Step 3: First Frame Render (`TutorialApp::view`)
Iced calls `view(&state)` to construct the initial widget tree, rasterizes text, computes layout geometry, and swaps the GPU buffer to display your UI on screen.

### Step 4: The Heartbeat Event Loop (99.9% of application runtime)
The application now enters a reactive event loop:

```text
 ┌──────────────────────────────────────────────────────────────┐
 │                      THE ICED EVENT LOOP                     │
 │                                                              │
 │  1. Sleep / Wait for an event:                               │
 │     • User clicks a button                                   │
 │     • User types a key                                       │
 │     • 1-second timer fires (from `TutorialApp::subscription`)│
 │     • Background async task finishes (from Tokio)            │
 │                                                              │
 │  2. Turn that event into a `Message`                         │
 │                                                              │
 │  3. Jump to `TutorialApp::update(&mut state, message)`       │
 │     └─► Mutate state (e.g. self.counter += 1)                │
 │                                                              │
 │  4. Jump to `TutorialApp::view(&state)`                      │
 │     └─► Repaint screen at 60+ FPS                            │
 └──────────────────────────────┬───────────────────────────────┘
                                │ (User clicks [X] to close window)
                                ▼
```

### Step 5: Clean Exit
When you close the window, the loop breaks, all memory and database handles are cleanly dropped via Rust's RAII (Resource Acquisition Is Initialization), and `.run()` finally returns `Ok(())` to `main()`.

---

## 🎯 Chapter 3: Function Pointers vs Function Calls (`foo` vs `foo()`)

In [`src/main.rs`], you see:
```rust
iced::application(
    TutorialApp::new,    // <-- NO PARENTHESES!
    TutorialApp::update, // <-- NO PARENTHESES!
    TutorialApp::view,   // <-- NO PARENTHESES!
)
```

### The Difference:
- **With parentheses `foo()`**: *"Execute this function RIGHT NOW on this line, and give me the result."*
- **Without parentheses `foo`**: *"Do NOT execute this function now. Here is a pointer/address to this function. Iced, YOU call it whenever you need to!"*

In JS, you do the exact same thing with event listeners:
```js
// CORRECT: Passing function reference:
button.addEventListener('click', handleClick);

// WRONG: Executes immediately on page load!
button.addEventListener('click', handleClick());
```