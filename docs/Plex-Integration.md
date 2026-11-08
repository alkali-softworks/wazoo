# Plex Media Server Metadata Integration Plan

This document outlines the architectural design, data models, network protocol, path-resolution algorithms, and user interface additions for integrating **Plex Media Server (PMS)** metadata into the **Wazoo** ambient media engine.

---

## 1. Vision & Objectives

Wazoo is an ambient multi-player and moving mood-board designed for artists, editors, and curators. While Wazoo currently relies on filename heuristic parsing (`format.rs`) to display titles, integrating Plex metadata elevates Wazoo into a semantic curation engine.

### Core Objectives
1. **Automated Rich Metadata Ingestion**: Fetch verified titles, series titles, seasons, episode numbers, release years, content ratings, plot synopses, genres, directors, and cast directly from Plex.
2. **Zero-Configuration Path Mapping**: Automatically correlate Plex's server-side filepaths (e.g., `/pub/movies/...` or `/volume1/...`) with Wazoo's local client mount paths (e.g., `/mnt/movies/...`) using suffix and subpath normalization.
3. **Multi-Dimensional Semantic Search**: Empower Wazoo's instant search modal to query not only raw filenames, but also release decades (`1980s`), specific years (`1999`), genres (`cyberpunk`, `sci-fi`, `horror`), directors (`tarantino`, `nolan`, `ghibli`), and plot keywords.
4. **Contextual HUD & Ambient Badges**: Present subtle, elegant release year chips, content ratings, genre tags, and plot synopsis tooltips within the player card overlays and Quick Picker.
5. **Non-Blocking Network Architecture**: Keep all network operations, sync passes, and image fetching entirely asynchronous via Tokio and background channels, ensuring 60+ FPS UI smoothness.

---

## 2. System Topology & Data Flow

```mermaid
flowchart TD
    subgraph NAS ["NAS / Remote Server (192.168.1.100)"]
        PMS["Plex Media Server (:32400)"]
        Filesystem["Remote Storage (/pub/...)"]
    end

    subgraph Client ["Client Machine (Wazoo Host)"]
        CIFS["Local Mount (/mnt/movies/...)"]
        
        subgraph Core ["wazoo-core"]
            PlexClient["Plex REST Client<br/>(reqwest + tokio)"]
            Matcher["Path Normalizer & Suffix Matcher"]
            SQLite[("wazoo.db<br/>(Video & VideoMetadata)")]
            Config["WazooSettings (settings.json)"]
        end

        subgraph App ["wazoo-app (Iced TEA)"]
            PlexTab["Settings Modal (Plex Tab)"]
            HUD["Player Card HUD / Overlays"]
            Search["Instant Search Modal"]
        end
    end

    Filesystem -.->|CIFS Mount| CIFS
    PlexClient -->|HTTP GET + X-Plex-Token| PMS
    PMS -->|Library Metadata JSON| PlexClient
    CIFS --> Matcher
    PlexClient --> Matcher
    Matcher -->|Batch Upsert| SQLite
    SQLite --> Search
    SQLite --> HUD
    PlexTab -->|Test & Trigger Sync| PlexClient
    Config --> PlexClient
```

---

## 3. Communication Protocol (Plex REST API)

Plex exposes a comprehensive REST API over HTTP/HTTPS on port `32400`. All requests from Wazoo pass standard Plex headers:

```http
X-Plex-Token: <USER_TOKEN>
X-Plex-Client-Identifier: wazoo-desktop-client
Accept: application/json
```

### Key Endpoints Used

| Endpoint | Purpose | Payload & Parsing |
| :--- | :--- | :--- |
| `GET /identity` | Connectivity & Version Verification | Returns `machineIdentifier`, `version`, and `claimed` status without requiring elevated scopes. |
| `GET /library/sections` | Discover Media Libraries | Returns list of library sections. Filtered for `type == "movie"` and `type == "show"`. Extracts library keys and server folder locations. |
| `GET /library/sections/{key}/all?includeGuids=1` | Bulk Fetch Movie Metadata | Returns all movies in a single JSON payload: `title`, `year`, `tagline`, `summary`, `contentRating`, `duration`, `rating`, `Genre`, `Director`, `Role`, `thumb`, `art`, and `Media.Part.file`. |
| `GET /library/sections/{key}/all?type=4` | Bulk Fetch TV Episodes | Setting `type=4` returns all episodes across all series in a single flat call: `grandparentTitle` (Show Name), `parentIndex` (Season), `index` (Episode), `title` (Episode Name), `year`, `summary`, `thumb`, and `Media.Part.file`. |
| `GET /library/metadata/{key}/thumb/{ts}` | High-Resolution Posters | On-demand poster thumbnail streaming, cached locally in `~/.cache/wazoo/posters/`. |

---

## 4. Path Resolution & Suffix Matching Algorithm

### The Discrepancy Problem
* **On NAS / Docker**: Files are indexed at `/pub/movies/Alien (1979)/Alien.mkv` or `/volume1/video/movies/...`.
* **In Wazoo Client**: Files are scanned locally at `/mnt/movies/movies/Alien (1979)/Alien.mkv`.
* Exact string matching fails because the root mount points differ.

### Multi-Tier Resolution Strategy
1. **Slash Normalization**: Convert all separators (`\` and `/`) to standard `/` and strip trailing slashes.
2. **Direct Exact Match**: Test if `plex_path == wazoo_path`.
3. **Library-Relative Match**:
   * Plex reports library root locations (e.g. `/pub/movies`).
   * Compute relative path inside Plex library: `Alien (1979)/Alien.mkv`.
   * Test if any configured Wazoo media folder contains `/Alien (1979)/Alien.mkv`.
4. **Hierarchical Suffix Match**:
   * Extract trailing path segments from the Plex path:
     - 1-segment: `Alien.mkv`
     - 2-segments: `Alien (1979)/Alien.mkv`
     - 3-segments: `movies/Alien (1979)/Alien.mkv`
   * Match against local indexed records in `Video(path)`.
   * Disambiguate multi-episode collisions using the 2+ segment parent folder.

```rust
pub fn match_plex_path(
    plex_path: &str,
    local_index: &HashMap<String, i64>,
    media_folders: &[String],
) -> Option<i64> {
    let norm_plex = plex_path.replace('\\', "/");

    // Tier 1: Exact
    if let Some(&id) = local_index.get(&norm_plex) {
        return Some(id);
    }

    // Tier 2: Suffix components (filename + parent folder)
    let segments: Vec<&str> = norm_plex.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() >= 2 {
        let suffix_2 = format!("{}/{}", segments[segments.len() - 2], segments[segments.len() - 1]);
        let candidates: Vec<i64> = local_index
            .iter()
            .filter(|(path, _)| path.replace('\\', "/").ends_with(&suffix_2))
            .map(|(_, &id)| id)
            .collect();

        if candidates.len() == 1 {
            return Some(candidates[0]);
        }

        // Tier 3: 3-segment check if disambiguation needed
        if segments.len() >= 3 && candidates.len() > 1 {
            let suffix_3 = format!(
                "{}/{}/{}",
                segments[segments.len() - 3],
                segments[segments.len() - 2],
                segments[segments.len() - 1]
            );
            if let Some((_, &id)) = local_index.iter().find(|(path, _)| path.replace('\\', "/").ends_with(&suffix_3)) {
                return Some(id);
            }
        }
    }

    None
}
```

---

## 5. Database Schema Extensions (`wazoo-core/src/db.rs`)

### New Table: `VideoMetadata`
A dedicated table storing structured metadata associated 1-to-1 with `Video(id)`:

```sql
CREATE TABLE IF NOT EXISTS VideoMetadata (
    video_id INTEGER PRIMARY KEY,
    plex_rating_key TEXT,
    title TEXT NOT NULL,
    series_title TEXT,
    season_number INTEGER,
    episode_number INTEGER,
    year INTEGER,
    tagline TEXT,
    summary TEXT,
    content_rating TEXT,
    duration_ms INTEGER,
    rating REAL,
    studio TEXT,
    genres TEXT,       -- JSON array of strings e.g. ["Action", "Sci-Fi"]
    directors TEXT,    -- JSON array of strings e.g. ["Ridley Scott"]
    actors TEXT,       -- JSON array of strings e.g. ["Sigourney Weaver", "Tom Skerritt"]
    thumb_url TEXT,    -- Remote Plex thumbnail endpoint
    art_url TEXT,      -- Remote Plex backdrop endpoint
    updated_at INTEGER,
    FOREIGN KEY (video_id) REFERENCES Video(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS meta_year_idx ON VideoMetadata(year);
CREATE INDEX IF NOT EXISTS meta_title_idx ON VideoMetadata(title);
CREATE INDEX IF NOT EXISTS meta_series_idx ON VideoMetadata(series_title);
```

### Enhanced Multi-Field Search
Wazoo's query parser in `db.rs` is upgraded with a `LEFT JOIN VideoMetadata m ON Video.id = m.video_id`:
* Positive query terms match against:
  - `Video.path` (existing)
  - `Video.name` (existing)
  - `m.title`
  - `m.series_title`
  - `m.genres` (e.g. `sci-fi`, `horror`)
  - `CAST(m.year AS TEXT)` (e.g. `1982`, `1999`)
  - `m.directors` (e.g. `kubrick`, `tarantino`)
  - `m.actors` (e.g. `harrison ford`)
  - `m.summary`

---

## 6. Domain Models & Configuration (`wazoo-core/src/models.rs`)

### 1. `MediaMetadata` Struct
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MediaMetadata {
    pub video_id: i64,
    pub plex_rating_key: Option<String>,
    pub title: String,
    pub series_title: Option<String>,
    pub season_number: Option<u32>,
    pub episode_number: Option<u32>,
    pub year: Option<u32>,
    pub tagline: Option<String>,
    pub summary: Option<String>,
    pub content_rating: Option<String>,
    pub duration_ms: Option<u64>,
    pub rating: Option<f32>,
    pub studio: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub directors: Vec<String>,
    #[serde(default)]
    pub actors: Vec<String>,
    pub thumb_url: Option<String>,
    pub art_url: Option<String>,
    pub updated_at: Option<u64>,
}
```

### 2. `PlexSettings` Struct
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlexSettings {
    pub enabled: bool,
    pub server_url: String,       // Default: "http://192.168.1.100:32400"
    pub token: String,            // User's X-Plex-Token
    pub auto_sync_on_scan: bool,  // Automatically sync when folder scan runs
    pub last_synced_at: Option<u64>,
}
```

---

## 7. User Interface Additions (`wazoo-app`)

### 1. Plex Tab in Settings Modal
A dedicated navigation button in the settings modal tabs bar: `[General] [Playback] [Filters] [CRT] [3D Cube] [Plex] [System]`. Clean text labels without emojis are used for optimal cross-language layout parity.

#### Controls within the Plex Tab:
* **Toggle Integration**: Enable/disable Plex metadata enrichment with live localized status badge (`Active` / `Off`).
* **Server URL Input**: Text input initialized to `http://192.168.1.100:32400`.
* **Plex Token Input**: Unmasked text input allowing direct verification and editing of the token, paired with an inline **Copy** button and a contextual hint (`Found in Plex Web XML or account settings`).
* **Test Connection Button**: Sends an asynchronous request to `/identity`. Displays server version, library name, and connection status.
* **Sync Now Button**: Triggers full library synchronization. Displays live metadata count (e.g., `19932 videos have enriched metadata`).
* **Auto-Sync Toggle**: "Sync Plex metadata automatically after folder scan".

### 2. Player Card HUD Enhancements
* **Rich Title Pill**:
  * Line 1: `Movie Title` or `Series Name`
  * Line 2: `S01E05 - Gray Matter` (for episodic series)
  * Badge Row: Subtle pills showing `[1999] [PG-13] [★ 8.7] [Sci-Fi] [Action]`
* **Plot Synopsis Tooltip**:
  * Hovering or clicking an info badge reveals a frosted backdrop card displaying the movie tagline, synopsis, director, and lead cast.

### 3. Quick Picker & Search Drawer
* Search result rows display the year and primary genre chip next to the title.
* Search syntax hints include examples: `anime, 1990s`, `sci-fi, not horror`, `nolan`.

### 4. Plex Explorer Modal (`E` Shortcut)
* **Keybinding `E`**: Toggles the 900px wide centered frosted-glass Plex Explorer modal from anywhere in the app.
* **Query Presets**:
  - `All`: Full library exploration across all indexed media, equipped with multi-dimensional sub-filters.
  - `Recently Added`: Pulls the newest additions across your library (live from `/library/recentlyAdded` or local cache).
  - `On Deck`: Displays continue-watching and in-progress movies/episodes with resume timestamps and progress bars (live from `/library/onDeck`).
  - `Top Rated`: Instant leaderboard of highest-rated films and shows (sorted by rating descending, then release year).
  - `Surprise Me`: Curated random shuffle of films with metadata. Every click on the `Surprise Me` tab immediately re-rolls a fresh random batch of up to 60 titles across your library, while keeping the list stable during search and browsing.
* **Simultaneous Genre & Decade Multi-Filtering (All Tab)**:
  - **Genres Row**: `[All] [Sci-Fi] [Horror] [Action] [Thriller] [Comedy] [Drama] [Crime] [Animation]`.
  - **Decades Row**: Placed directly below Genres: `[All] [80s] [90s] [00s] [10s] [20s]`.
  - Users can combine any genre and decade simultaneously on the `All` tab (e.g., **80s Horror**).
  - Clicking an active filter toggles it off, or clicking `[All]` clears that filter. Toggling filters preserves the current tab without jumping.
  - **Robust Genre Normalization**: Standardizes Plex's `"Science Fiction"` category with `"Sci-Fi"` / `"scifi"` aliases across both filter buttons and live instant text search, correctly matching all cataloged films.
* **Isolated Filter Scope**:
  - The `Genres:` and `Decades:` filter controls are isolated strictly to the **All** tab. Preset tabs (`Top Rated`, `Recently Added`, `On Deck`, and `Surprise Me`) remain uncluttered and focused on their specialized feeds.
* **Optimized Card Typography & Badges**:
  - Movie titles occupy line 1 alongside right-aligned badges (`Year`, `Content Rating`, `★ Rating`, `Genres`). Badges are guaranteed natural width and never get clipped.
  - TV episodes display season and episode designations (`S01E02 - Episode Name`) next to the series title.
  - Taglines and plot summaries are displayed on line 2 with automatic deduplication.
* **Live Instant Search**: Integrated filter input querying titles, subtitles, directors, actors, genres, years, and summaries.
* **One-Click Play & Resume**: Clicking any item immediately begins playback in the focused player, automatically seeking to any saved `viewOffset` from Plex.

---

## 8. Implementation Milestones & Status

```mermaid
flowchart TD
    M1["Phase 1: Core Models & DB<br/>✓ MediaMetadata, VideoMetadata table, SQL multi-field search"] --> M2["Phase 2: Plex REST Client<br/>✓ Reqwest async client, /identity check, path matcher"]
    M2 --> M3["Phase 3: Settings UI & Sync<br/>✓ Plex settings tab, unmasked token, copy button, 19k+ sync"]
    M3 --> M4["Phase 4: Plex Explorer Modal<br/>✓ 'E' key shortcut, 900px modal, 'All' tab, 80s horror multi-filtering"]
    M4 --> M5["Phase 5: i18n & Verification<br/>✓ All 23 languages translated, 139 workspace tests passing"]
```

| Phase | Milestone | Deliverables | Status |
| :--- | :--- | :--- | :--- |
| **Phase 1** | **Data Models & Schema** | Add `MediaMetadata` & `PlexSettings` models. Create `VideoMetadata` table in `wazoo-core/src/db.rs`. Extend `search_videos` to query metadata fields. | **Completed** (`dcbc38e`) |
| **Phase 2** | **Plex Client & Sync Engine** | Implement `wazoo-core/src/plex.rs` with `reqwest`. Add `/identity`, `/library/sections`, `/library/recentlyAdded`, `/library/onDeck`, and bulk `/all` ingestion. Implement path suffix matcher. | **Completed** (`dcbc38e`) |
| **Phase 3** | **UI Settings & Sync Flow** | Add `SettingsTab::Plex` in `wazoo-app`. Implement unmasked token with Copy button, Test Connection, and live sync (19,932 items indexed in seconds). | **Completed** (`23189de`, `873f84e`) |
| **Phase 4** | **Plex Explorer Modal (`E`)** | Implement 900px wide modal, `[All]` preset, simultaneous genre & decade filtering (e.g. 80s Horror), Science Fiction / Sci-Fi normalization, filter isolation to All tab, and one-click play/resume. | **Completed** (`927c15a`, `9b07b08`, `4ee4992`, `35b8005`, `dcf4277`) |
| **Phase 5** | **i18n & Test Verification** | Translate all Plex keys across all 23 languages with 100% key parity. 140 tests passing across workspace. | **Completed** (`5724171`, `35b8005`, `dcf4277`) |

