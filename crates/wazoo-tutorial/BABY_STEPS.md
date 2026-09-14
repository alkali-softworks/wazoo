# 🦀 Rust for PHP & JS Survivors: Baby Steps Guide

*A no-nonsense handbook for understanding Rust & Iced.*

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

---

## 🧱 Chapter 4: There Are No Classes in Rust! (`struct`, `impl`, and `let mut`)

In PHP and JS, Object-Oriented Programming glues your **data** and your **methods** together inside a `class`:

```php
// PHP: Data and methods glued together in one class
class User {
    public $name;
    public $age;

    public function celebrateBirthday() {
        $this->age++;
    }
}
```

**Rust has NO `class` keyword.**
Instead, Rust separates the **DATA** from the **BEHAVIOR**:
1. **`struct`** defines the shape of the data (like a database schema or TypeScript interface).
2. **`impl`** defines the functions and methods that operate on that data.

```rust
// 1. DATA: Just the fields in memory
pub struct User {
    pub name: String,
    pub age: u32,
}

// 2. BEHAVIOR: "impl" = "Implement methods for User"
impl User {
    // "new" is NOT a keyword or special constructor!
    // It is literally just a regular function name by convention.
    // You could name it `create`, `build`, or `spawn`!
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }

    // A method on an instance (&mut self)
    pub fn celebrate_birthday(&mut self) {
        self.age += 1;
    }
}
```

---

### ❓ "Wait, if it's not a class, how does it have a constructor? Do I say `new User('klo', 42)`?"

**NO! There is NO `new` keyword in Rust!**

In JS and PHP, `new` is a built-in language operator that allocates an object on the heap and triggers `__construct()`:
```js
// JavaScript:
const user = new User("klo", 42);
```
```php
// PHP:
$user = new User("klo", 42);
```

**In Rust, `new` is NOT a language keyword.**
Rust doesn't have constructors at the language level. Instead, there are **two ways** to create a struct:

#### Method A: The Conventional Function Call (`User::new(...)`)
Because `new` is just a regular static function in `impl User`, you call it using double colons `::`:
```rust
// How you call it in Rust:
let mut user = User::new(String::from("klo"), 42);

// Now you can call methods on it with a dot:
user.celebrate_birthday();
println!("Age is now: {}", user.age); // 43
```
*(Notice `String::from("klo")` because `"klo"` in quotes is a borrowed `&str`, but our struct owns a heap `String`. And `42` is an integer number, not a string `'42'`!)*

#### Method B: Direct Struct Literal (No function needed at all!)
If a struct's fields are public, you don't even need a `new()` function! You can instantiate it directly in place (like a JS object literal):
```rust
let user = User {
    name: String::from("klo"),
    age: 42,
};
```

So why do Rust developers write `pub fn new(...)`?
1. **Validation**: You can enforce rules (e.g. `if age < 0 { return Err(...); }`).
2. **Encapsulation**: If fields are private, `new()` is the only gatekeeper to construct the struct.
3. **Defaults**: You can set initial default values for fields so the caller doesn't have to specify all 15 fields manually.


### 🔒 `let` vs `let mut` (Immutability by Default)

In PHP, all variables are mutable `$x = 1; $x = 2;`.
In JS, you have `const` and `let`.

**In Rust, EVERYTHING is `const` by default!**

```rust
let x = 5;
x = 6; // 💥 COMPILE ERROR! `x` is immutable!
```

If you want to be able to change a variable's value, you must **explicitly** write `mut` (short for mutable):

```rust
let mut x = 5;
x = 6; // ✅ Perfectly fine because you opted into mutability.
```

Why does Rust do this?
Because accidental variable mutation is one of the biggest sources of bugs and race conditions in multi-threaded programs. In Rust, you can look at any variable and know: *if it doesn't say `mut`, its value will never change.*

---

### 👤 `self` vs `Self` (Lowercase vs Uppercase)

Inside an `impl` block, you will see both:

1. **`Self`** (Capital `S`):
   - An alias for the **TYPE** name.
   - Inside `impl TutorialApp`, `Self` literally means `TutorialApp`.
   - `-> Self` means "this function returns a `TutorialApp`".
   - `Self::new()` means `TutorialApp::new()`.

2. **`self`** (Lowercase `s`):
   - The actual **INSTANCE** of the object!
   - In PHP, this is `$this`.
   - In JS, this is `this`.

---

### ⚙️ Static Functions vs Instance Methods

How does Rust know if a function is static (called on the class) or an instance method (called on an object)?
Look at the **first argument**:

```rust
impl TutorialApp {
    // STATIC FUNCTION (No `self` in arguments):
    // Like `public static function new()` in PHP.
    // Called with double-colons: TutorialApp::new()
    pub fn new() -> (Self, Task<Message>) {
        ...
    }

    // READ-ONLY METHOD (Takes `&self`):
    // Borrows read access to `$this`.
    // Called with dot: app.view()
    pub fn view(&self) -> Element<'_, Message> {
        ...
    }

    // MUTABLE METHOD (Takes `&mut self`):
    // Borrows write access to `$this`.
    // Called with dot: app.update(msg)
    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.counter_value += 1; // Can mutate fields!
        ...
    }
}
```

#### ❓ "Do I need to pass in `self` when calling `app.view()`?"

**NO! You do NOT pass `self`!**

You just call it with the dot operator like in JS/PHP:
```rust
app.view();                // <-- NO arguments passed for self!
app.update(my_message);    // <-- Only pass the other arguments!
```

The dot operator `.` is syntactic sugar. Behind the scenes, the compiler automatically passes `&app` or `&mut app` as the first argument:

| Method Type | How You Define It | How You Call It | What Compiler Does Behind the Scenes |
| :--- | :--- | :--- | :--- |
| **Static Function** | `fn new() -> Self` | `TutorialApp::new()` | `TutorialApp::new()` |
| **Read-Only Method** | `fn view(&self)` | `app.view()` | `TutorialApp::view(&app)` |
| **Mutable Method** | `fn update(&mut self, msg)` | `app.update(msg)` | `TutorialApp::update(&mut app, msg)` |
| **Consuming Method** (takes ownership) | `fn close(self)` | `app.close()` | `TutorialApp::close(app)` |

#### 🎯 The PHP Mental Model: `::` vs `.`

Think of it exactly like PHP:
```
In PHP:
  App::new()      <-- Double colons `::` for STATIC call on the class
  $app->view()    <-- Arrow `->` for INSTANCE call on the object

In Rust:
  TutorialApp::new()  <-- Double colons `::` for STATIC call on the type
  app.view()          <-- Dot `.` for INSTANCE call on the object
```
*(The only visual difference is Rust uses a dot `.` where PHP uses `->`, because in PHP the dot was already reserved for string concatenation `$a . $b`!)*

Whenever you see:
- **`::`** (double colon) $\rightarrow$ You are calling the **type/namespace** (static).
- **`.`** (dot) $\rightarrow$ You are calling the **instance** of an object.


### Cheat Sheet: PHP/JS vs Rust

| Concept | PHP / JS | Rust |
| :--- | :--- | :--- |
| Object shape | `class User { public $name; }` | `struct User { name: String }` |
| Attach methods | Inside the `class` block | Inside `impl User { ... }` |
| Immutable variable | `const x = 5;` | `let x = 5;` |
| Mutable variable | `$x = 5;` / `let x = 5;` | `let mut x = 5;` |
| `$this` / `this` | `$this` / `this` | `self` |
| Class Name Alias | `self::` (PHP) | `Self` |
| Static call | `User::create()` | `User::create()` |
| Method call | `$user->save()` / `user.save()` | `user.save()` |

---

### 💡 "WTF is `let app = Self { ... }` in `app.rs` line 122?"

When you see:
```rust
impl TutorialApp {
    pub fn new() -> (Self, Task<Message>) {
        ...
        let app = Self {
            counter_value: 0,
            active_tab: Tab::CounterAndWidgets,
            ...
        };

        (app, Task::none())
    }
}
```

It looks like the function is referencing itself in a circle, but here is what is actually happening:

1. **`Self` is an alias for `TutorialApp`**:
   Because we are inside `impl TutorialApp`, writing `Self { ... }` is 100% identical to writing `TutorialApp { ... }`.
2. **It constructs a new instance**:
   In PHP: `$app = new self();`
   In JS: `const app = new TutorialApp();`
3. **Implicit Return (No `return` keyword, no semicolon!)**:
   Notice line `(app, Task::none())` at the bottom of `new()`.
   In Rust, **the last expression in a function without a semicolon `;` is automatically returned**!
   You don't need to write `return (app, Task::none());`. Leaving off the semicolon makes it the return value.

---

## 🎯 Chapter 5: Pattern Matching (`match`), Arrows (`=>` vs `->`), & Destructuring

When looking at `app.rs`:
```rust
pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::TabSelected(tab) => {
            self.active_tab = tab;
            ...
        }
    }
}
```

### 1. The Two Arrows in Rust (`->` vs `=>`)

Don't confuse them:
- **`->` (Thin Arrow)**: Used exclusively for **Function Return Types**.
  - `fn update(...) -> Task<Message>` means *"this function returns a `Task<Message>`"*.
- **`=>` (Fat Arrow)**: Used in **`match` expressions**.
  - It means: *"IF THIS PATTERN MATCHES $\implies$ EXECUTE THIS CODE"*.

---

### 2. How `match` Replaces Ugly `switch` Statements

In JS or old PHP, handling events looked like:
```js
// JavaScript switch:
switch (message.type) {
    case 'TabSelected':
        const tab = message.payload; // Manual unpacking
        this.activeTab = tab;
        break; // Forgot break? Bug!
}
```

In Rust, `match` does pattern matching and variable extraction in one shot:
```rust
Message::TabSelected(tab) => {
// ^^^^^^^^^^^^^^^^^ ^^^  ^^
//        1           2    3
```
1. **Match Variant**: Checks if `message` is the `TabSelected` variant.
2. **Destructure / Unpack**: `(tab)` creates a brand-new local variable holding the payload data!
3. **`=>`**: Executes the block on the right.

---

### 3. Exhaustiveness: Why You Can't Forget Cases

In JS and PHP, if you add a new event to your app and forget to update your `switch`, the code runs and silently fails.

**In Rust, `match` is 100% EXHAUSTIVE.**
If your `enum Message` has 15 variants, your `match` MUST handle all 15. If you miss even one, the compiler stops you:
```
error[E0004]: non-exhaustive patterns: `Message::ResetCounter` not covered
```
This is why Rust refactoring feels like magic: when you change a feature, the compiler gives you a complete todo list of every place that needs updating!

---

### 4. Unit Variants vs Data Variants & `Task::none()`

In `app.rs`:
```rust
Message::Tick => {
    self.uptime_seconds += 1;
    Task::none()
}
```

Notice two big differences from `TabSelected(tab)`:

1. **No Parentheses (Unit Variant)**:
   - `Message::TabSelected(Tab)` carries data (a `Tab`).
   - `Message::Tick` carries **no data**. In Rust, an enum variant with no data is called a **Unit Variant**. It acts like a pure event trigger or signal (no payload to unpack).

2. **Subscriptions vs `setInterval()`**:
   - In JS, you'd do: `setInterval(() => { this.seconds++ }, 1000)`. It directly mutates state whenever the timer fires.
   - In Iced, all state changes must pass through `update()`. You declare a `Subscription` on line 185: `time::every(1s).map(|_| Message::Tick)`. Every second, Iced sends a `Message::Tick` into the main event loop!

3. **What is `Task::none()`?**:
   - `update()` returns `Task<Message>`. A `Task` tells Iced: *"Go do background async work on Tokio (like an HTTP request or file dialog), then send me another Message when done."*
   - Returning `Task::none()` simply means: *"I changed state in memory. No background async tasks needed!"*

---

## 🪈 Chapter 6: Closures (`|x|`), The Underscore `_`, & `.map()`

On line 186 in `app.rs`, you see this:
```rust
pub fn subscription(&self) -> Subscription<Message> {
    time::every(Duration::from_secs(1)).map(|_| Message::Tick)
}
```

### 1. Pipes `| |` are Arrow Functions!

In JavaScript and PHP, you write arrow functions like:
```js
// JavaScript:
(x) => x * 2
```
```php
// PHP:
fn($x) => $x * 2
```

In Rust, closures use **pipes `| |`** around the parameters:
```rust
// Rust:
|x| x * 2
```

Why pipes? Because Rust already uses parentheses `(x, y)` for **Tuples**. To keep the syntax unambiguous, closures use pipes!

```rust
// Multi-argument closure:
let add = |a, b| a + b;
println!("{}", add(10, 20)); // 30
```

---

### 2. What Does the Underscore `_` Mean?

In Rust, the underscore `_` means: **"Ignore this parameter."**

`time::every(...)` actually produces an `Instant` (the exact clock timestamp when the timer ticked).
We don't need the timestamp—we just want to know that 1 second passed!
- If you wrote `|timestamp| Message::Tick`, Rust would complain: `warning: unused variable: timestamp`.
- Writing `|_|` tells Rust: *"Yes, I know a timestamp is being passed here, but intentionally discard it."*

---

### 3. What is `.map(...)` Doing?

Just like `array.map()` in JavaScript or `array_map()` in PHP:
- `time::every(...)` produces a stream of timestamps: `Subscription<Instant>`.
- But Iced requires our function to return a stream of messages: `Subscription<Message>`.
- `.map(|_| Message::Tick)` intercepts each tick and converts it into `Message::Tick`!

---

## 👁️ Chapter 7: Scoped Visibility (`pub` vs `pub(crate)` vs Private)

In `wazoo-app/src/app.rs`, you see fields defined like:
```rust
pub struct WazooApp {
    pub(crate) db: Database,
    pub(crate) settings: WazooSettings,
    pub(crate) players: Vec<VideoHandle>,
}
```

In PHP, you only have three blunt options: `public`, `protected`, and `private`.

Rust gives you fine-grained **scoped visibility** to build clean, leak-proof architectures:

### 1. The Problem with Just `pub` and `private`

- **If you leave it private (no keyword)**: 
  Only code inside `src/app.rs` could touch `self.db`. Other internal files like `src/update.rs` or `src/views/history.rs` would get compile errors: *"Cannot access private field `db`"*.
- **If you make it `pub`**: 
  Now it's public to the entire outside world! Anyone who imports `wazoo-app` could reach in and corrupt your database handle directly.

### 2. The Solution: `pub(crate)` (Package-Internal Visibility)

Writing:
```rust
pub(crate) db: Database
```
Tells the compiler:
> *"Make this field accessible to **any file inside `crates/wazoo-app`**, but keep it **strictly hidden (private)** to any outside code or crates!"*

In **JavaScript / npm** terms:
Think of an npm library. You might have 20 internal helper files that import each other, but your `package.json` only exports 1 clean class in `index.js`.
`pub(crate)` is for all those internal files to talk to each other without exposing those guts to people who `npm install` your package!

In **PHP** terms:
PHP has always lacked this! In PHP, if you make a method `public` so another file in your Composer library can call it, you've accidentally made it public to your library's consumers too. Rust's `pub(crate)` solves this: public inside the package, private outside.

### 3. Visibility Cheat Sheet

| Syntax | Scope | In Plain English |
| :--- | :--- | :--- |
| `field: Type` *(no keyword)* | File / Module | **`private`**: Only code in this exact file can see it |
| `pub(crate) field: Type` | Entire Crate | **Package-Internal**: Any file in this crate can see it, but outside crates can't |
| `pub(super) field: Type` | Parent Module | **Parent-Only**: Only the parent folder/module (`../`) can see it |
| `pub field: Type` | Entire Universe | **`public`**: Anyone who imports this crate can see it |

---

## 🎁 Chapter 8: The Death of `null` — Understanding `Option`, `Some`, & `None`

In `wazoo-app/src/update.rs`, you see code like:
```rust
if let Some(player) = self.focused_player_mut() {
    player.set_subtitles_visible(true);
}
```

This is the solution to what Tony Hoare (the inventor of `null`) famously called his **"Billion Dollar Mistake"**.

---

### 1. Why `null` Sucks in PHP and JS

In PHP and JavaScript, when something might not exist, functions return `null` or `undefined`:
```php
// PHP:
$player = $this->getFocusedPlayer(); 
// If no player is focused, $player is null!
// If you forget to check, your whole app CRASHES in production:
// "Fatal Error: Call to a member function set_subtitles_visible() on null"
$player->set_subtitles_visible(true); 
```

```js
// JavaScript:
const player = this.getFocusedPlayer();
// "TypeError: Cannot read properties of undefined (reading 'set_subtitles_visible')"
player.set_subtitles_visible(true);
```

---

### 2. Rust's Solution: Zero Nulls, Ever

Rust **completely deleted `null` and `undefined` from the language**.

If a value might be missing, Rust forces the function to return an enum called **`Option<T>`**:

```rust
enum Option<T> {
    Some(T),  // ✅ The value exists! It's wrapped safely inside `Some`.
    None,     // ❌ The value doesn't exist.
}
```

Because `focused_player_mut()` returns `Option<&mut VideoHandle>`, the compiler **physically forbids** you from calling methods on it directly:
```rust
self.focused_player_mut().set_subtitles_visible(true); // 💥 COMPILE ERROR!
// error[E0599]: no method named `set_subtitles_visible` found for enum `Option`
```
The compiler protects you: *"You cannot touch the player until you check if it's actually there!"*

---

### 3. How to Unpack It: `if let Some(...)`

To get the player out of the `Option` wrapper, you write:
```rust
if let Some(player) = self.focused_player_mut() {
    player.set_subtitles_visible(true);
}
```

In plain English, this means:
> *"If `focused_player_mut()` returned a **`Some`**, unwrap the inner value, name it **`player`**, and run the `{ ... }` block. If it returned **`None`**, do nothing and safely skip the block!"*

---

### 4. Comparison Table

| Scenario | In PHP / JS | In Rust |
| :--- | :--- | :--- |
| Value exists | `$val = "hello"` | `Some("hello")` |
| Value missing | `$val = null` / `undefined` | `None` |
| Checking for value | `if ($val !== null)` | `if let Some(val) = ...` |
| If you forget to check | 💥 App crashes at runtime | 🛡️ **Compile error!** Impossible to forget. |