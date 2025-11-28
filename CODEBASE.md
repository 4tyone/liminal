# TutorGPT Codebase Guide

A complete walkthrough of the codebase structure, what each file does, and what's implemented vs. placeholder.

---

## Project Structure

```
TutorGPT/
├── index.html                 # Entry point for Vite/Tauri
├── vite.config.ts             # Vite dev server config
├── package.json               # Node dependencies
├── tailwind.config.js         # (not used - Tailwind v4 uses input.css)
│
├── src/                       # Frontend source
│   ├── input.css              # Tailwind + custom styles
│   ├── output.css             # Generated CSS (don't edit)
│   └── js/
│       ├── app.js             # Main entry + mock API
│       ├── api.js             # Tauri command wrappers
│       ├── router.js          # Hash-based SPA router
│       ├── markdown.js        # Markdown rendering
│       ├── pages/
│       │   ├── settings.js    # API key page
│       │   ├── projects.js    # Project gallery
│       │   └── reader.js      # Learning reader
│       └── components/
│           ├── toast.js       # Notifications
│           └── selection-popover.js  # Text selection UI
│
└── src-tauri/                 # Rust backend
    ├── Cargo.toml             # Rust dependencies
    ├── tauri.conf.json        # Tauri config
    └── src/
        ├── main.rs            # Binary entry (calls lib::run)
        ├── lib.rs             # Tauri setup + command registration
        ├── models/
        │   ├── mod.rs         # Module exports
        │   ├── project.rs     # Project structs
        │   ├── page.rs        # Page struct
        │   └── expansion.rs   # Selection/expansion structs
        ├── services/
        │   ├── mod.rs         # Module exports
        │   ├── file_service.rs    # File I/O operations
        │   ├── config_service.rs  # Config persistence
        │   └── ai_service.rs      # ⚠️ AI PLACEHOLDERS
        └── commands/
            ├── mod.rs         # Module exports
            ├── config.rs      # Config commands
            ├── projects.rs    # Project/page commands
            └── ai.rs          # AI commands
```

---

## Frontend Deep Dive

### `index.html`

The entry point loaded by Vite/Tauri. Contains:
- CSS link to Tailwind output
- Three main elements: `#app`, `#selection-popover`, `#loading-overlay`, `#toast`
- Script tag loading `app.js`

### `src/input.css`

Tailwind v4 configuration + custom book-inspired styles:

```css
@import "tailwindcss";
@plugin "@tailwindcss/typography";

:root {
  /* Warm aged paper colors */
  --color-bg: #f4f1eb;
  --color-bg-elevated: #faf8f4;
  --color-text-primary: #2c2416;
  --color-accent: #2c2416;

  /* Book typography */
  --font-serif: 'Georgia', 'Times New Roman', Palatino, serif;
  --font-sans: -apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif;
  --font-mono: 'Courier New', Courier, monospace;

  /* ... more variables */
}
```

**Design Philosophy:** Book-like aesthetic with warm paper colors, serif typography, justified text, paragraph indentation, and drop caps.

**Custom component classes:**
- `.btn`, `.btn-primary`, `.btn-secondary` - Black button styles
- `.input` - Form input styles
- `.card` - Card container styles (no hover animations)
- `.toast`, `.toast-success`, `.toast-error` - Notification styles
- `.popover` - Text selection popover
- `.ai-expansion` - Margin note style AI content blocks
- `.content-flash` - Flash animation for updated content
- `.prose` - Book-style typography with justified text, indentation, drop caps
- `.reader-container` - Centered book page layout (800px width)
- `.loading-state`, `.loading-overlay` - Loading indicators

**Visual Effects:**
- Paper texture overlay (SVG noise at 2.5% opacity)
- Subtle vignette effect on edges
- No hover animations (removed for cleaner feel)

**To rebuild CSS:** `npx @tailwindcss/cli -i src/input.css -o src/output.css`

---

### `src/js/app.js`

Main application entry point.

**What it does:**
1. Imports and registers routes with the router
2. Checks for API key on startup
3. Redirects to settings if no key, otherwise starts router
4. Provides mock API for development without Tauri

**Mock API (lines 36-147):**
When running in browser without Tauri, provides fake implementations:
- `get_api_key` / `set_api_key` - Uses localStorage
- `list_projects` - Returns demo project
- `get_project` - Returns demo project data
- `get_page_content` - Returns sample markdown
- `generate_learning` - Returns new project stub
- `expand_selection` - Simulates AI expansion insertion

---

### `src/js/api.js`

Tauri IPC wrapper functions. Each function calls `window.__TAURI__.core.invoke()`:

| Function | Rust Command | Purpose |
|----------|--------------|---------|
| `getApiKey()` | `get_api_key` | Retrieve stored API key |
| `setApiKey(key)` | `set_api_key` | Save API key |
| `listProjects()` | `list_projects` | Get all projects |
| `getProject(id)` | `get_project` | Get single project |
| `createProject(title, desc)` | `create_project` | Create empty project |
| `deleteProject(id)` | `delete_project` | Delete project |
| `getPageContent(projectId, pageName)` | `get_page_content` | Read page markdown |
| `savePageContent(...)` | `save_page_content` | Write page markdown |
| `addPage(projectId, title)` | `add_page` | Add new page |
| `reorderPages(projectId, order)` | `reorder_pages` | Change page order |
| `generateLearning(topic, depth)` | `generate_learning` | ⚠️ AI: Generate content |
| `expandSelection(...)` | `expand_selection` | ⚠️ AI: Expand selection |
| `removeExpansion(...)` | `remove_expansion` | Remove AI expansion |

---

### `src/js/router.js`

Simple hash-based router for SPA navigation.

**How it works:**
- Listens to `hashchange` events
- Matches URL hash against registered routes
- Supports parameters (e.g., `/project/:id`)
- Calls registered handler function with params

**Usage:**
```javascript
router.register('/settings', renderSettings);
router.register('/project/:id', renderReader);
router.navigate('/projects');
```

---

### `src/js/markdown.js`

Markdown rendering with syntax highlighting.

**Dependencies:** `marked` (markdown parser), `highlight.js` (syntax highlighting)

**Functions:**
- `renderMarkdown(content)` - Converts markdown to HTML
- `findTextInMarkdown(markdown, selectedText)` - Finds line numbers for selected text
  - Normalizes whitespace for better matching
  - Handles single-line selections
  - Handles multi-line selections (up to 20 lines)
  - Returns `{ startLine, endLine, found }`
- `insertAfterLine(markdown, lineNumber, content)` - Inserts content after line

**Recent Improvements:**
- Multi-line selection now properly detects text spanning multiple lines
- Whitespace normalization handles formatting differences
- Returns accurate start and end line numbers for backend processing

---

### `src/js/pages/settings.js`

API key configuration page.

**UI:**
- Password input for API key
- Save button
- Back button (to projects)

**Flow:**
1. On load, fetches existing API key
2. On save, validates input and stores via Tauri
3. Redirects to projects on success

---

### `src/js/pages/projects.js`

Project gallery and creation page.

**UI:**
- Header with settings button
- Grid of project cards (title, description, page count, date)
- Delete button on each card
- "Create New Learning" form with topic input and depth selector

**Functions:**
- `renderProjects()` - Main render function
- `loadProjects()` - Fetches and displays project list
- `handleGenerate()` - Triggers AI generation (calls `generateLearning`)

**⚠️ AI Integration:** The "Generate Learning Material" button calls `generateLearning()` which currently returns placeholder content.

---

### `src/js/pages/reader.js`

Markdown reader with page navigation and text selection.

**UI:**
- Topbar with back button, project title, and page navigation
- Centered book-style content area (800px width)
- Markdown rendered with prose styles (justified text, indentation, drop caps)

**Functions:**
- `renderReader(params)` - Main render, sets up selection handler
- `loadProject(projectId)` - Loads project metadata
- `loadCurrentPage()` - Loads and renders current page content
- `navigatePage(delta)` - Page navigation
- `handleAskQuestion(selectedText, question)` - AI expansion handler
- `flashChangedLines(oldLines, newLines, contentEl, updatedLineNumbers)` - Flashes updated content

**Selection Flow (Updated):**
1. User selects text in `#markdown-content`
2. `selection-popover.js` shows popover
3. User enters question
4. `handleAskQuestion()` called
5. Frontend finds line numbers (e.g., 12-17) using `findTextInMarkdown()`
6. Sends `{ startLine, endLine, selectedText, question }` to backend
7. Backend (AI logic) processes and returns updated markdown
8. Frontend re-renders and flashes changed content with warm highlight animation
9. Automatically scrolls to show the changes

**Flash Animation:**
- Compares old vs new content to detect changes
- Flashes last 5 elements by default (where new content typically appears)
- Backend can optionally return `updatedLines: [array]` to specify exact elements
- Warm tan highlight with subtle glow (1.5s duration)

---

### `src/js/components/toast.js`

Simple toast notification system with book-inspired styling.

**Functions:**
- `showToast(message, type)` - Show notification
- `showSuccess(message)` - Success toast (green)
- `showError(message)` - Error toast (red)

**Styling:** Black background, white text, slides up from bottom center
Toasts auto-dismiss after 3 seconds.

---

### `src/js/components/loading.js`

Loading overlay component.

**Functions:**
- `showLoading(message)` - Show full-screen loading overlay with message
- `hideLoading()` - Hide loading overlay

**Styling:** Warm paper background with blur, spinning indicator

---

### `src/js/components/selection-popover.js`

Text selection UI for asking AI questions.

**Functions:**
- `initSelectionPopover(onAsk)` - Initialize with callback
- `cleanupSelectionPopover()` - Remove event listeners
- `hidePopover()` - Hide and reset

**UI Elements:**
- Selected text preview (italic serif font, truncated to 60 chars)
- Question input field
- Submit button (arrow icon)

**Event Handling:**
- `mouseup` / `keyup` - Detect selection end
- Validates selection is within `#markdown-content`
- Ignores selections < 3 characters
- Escape key to close
- Enter key to submit

---

## Backend Deep Dive

### `src-tauri/Cargo.toml`

Rust dependencies:

| Crate | Version | Purpose |
|-------|---------|---------|
| `tauri` | 2 | Desktop app framework |
| `tauri-plugin-opener` | 2 | Open URLs in browser |
| `serde` | 1 | Serialization |
| `serde_json` | 1 | JSON handling |
| `tokio` | 1 | Async runtime |
| `uuid` | 1 | Generate IDs |
| `dirs` | 5 | App data directories |
| `chrono` | 0.4 | Date/time handling |
| `slug` | 0.1 | URL-safe slugs |
| `regex` | 1 | Pattern matching |
| `pocketflow_rs` | 0.1 | ⚠️ AI orchestration (unused) |

---

### `src-tauri/src/lib.rs`

Tauri application setup.

**Registered Commands:**
```rust
tauri::generate_handler![
    // Config
    get_api_key,
    set_api_key,
    // Projects
    list_projects,
    get_project,
    create_project,
    delete_project,
    // Pages
    get_page_content,
    save_page_content,
    add_page,
    reorder_pages,
    // AI
    generate_learning,
    expand_selection,
    remove_expansion,
]
```

---

### `src-tauri/src/models/`

Data structures used throughout the app.

**`project.rs`:**
```rust
pub struct ProjectMeta {
    pub id: String,
    pub title: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub page_order: Vec<String>,
}

pub struct ProjectListItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub page_count: usize,
    pub updated_at: DateTime<Utc>,
}
```

**`page.rs`:**
```rust
pub struct Page {
    pub name: String,
    pub title: String,
}
```

**`expansion.rs`:**
```rust
pub struct SelectionRange {
    pub start_line: usize,
    pub end_line: usize,
    pub selected_text: String,
}

pub struct ExpansionResult {
    pub expansion_id: String,
    pub updated_markdown: String,
    pub inserted_content: String,
    pub insertion_line: usize,
}
```

---

### `src-tauri/src/services/file_service.rs`

All file system operations.

**Directory Functions:**
- `get_app_data_dir()` - Returns `~/Library/Application Support/TutorGPT/`
- `get_projects_dir()` - Returns `.../TutorGPT/projects/`
- `get_project_dir(id)` - Returns `.../projects/{id}/`

**Project Functions:**
- `list_all_projects()` - Reads all project meta.json files
- `load_project(id)` - Reads single project meta.json
- `save_project(meta)` - Writes project meta.json
- `delete_project_dir(id)` - Removes project directory
- `create_new_project(title, desc)` - Creates new project with slug ID

**Page Functions:**
- `load_page_content(project_id, page_name)` - Reads markdown file
- `save_page_content(project_id, page_name, content)` - Writes markdown file
- `add_page_to_project(project_id, title, content)` - Creates new page file

---

### `src-tauri/src/services/config_service.rs`

Configuration persistence.

**Config Structure:**
```rust
pub struct Config {
    pub api_key: Option<String>,
    pub theme: String,
}
```

**Functions:**
- `load_config()` - Reads config.json
- `save_config(config)` - Writes config.json
- `get_api_key()` - Gets just the API key
- `set_api_key(key)` - Sets just the API key

**Storage:** `~/Library/Application Support/TutorGPT/config.json`

---

### `src-tauri/src/services/ai_service.rs`

⚠️ **THIS FILE CONTAINS PLACEHOLDERS - YOU MUST IMPLEMENT**

**`generate_learning_material(topic, depth, api_key)`**

Currently:
- Creates empty project with slug from topic
- Adds single placeholder introduction page
- Returns project metadata

Should:
1. Use PocketFlow to research topic
2. Generate structured outline
3. Create multiple content pages
4. Return populated project

**`expand_selection_with_ai(project_id, page_name, selection, question, api_key)`**

Currently:
- Loads page content
- Generates placeholder expansion with `<details>` HTML
- Inserts after selected line
- Saves and returns result

**Frontend now sends:**
```rust
{
  startLine: usize,      // e.g., 12
  endLine: usize,        // e.g., 17
  selectedText: String,  // The actual selected text
  question: String       // User's question
}
```

Should:
1. Receive line numbers and selected text
2. Load full page for context
3. Use PocketFlow to generate contextual answer
4. Update markdown at appropriate location (AI decides where)
5. Optionally return `updatedLines: Vec<usize>` to indicate which line elements to flash
6. Return `{ updatedMarkdown: String, updatedLines?: Vec<usize> }`

**Note:** The AI backend is responsible for determining where to insert/modify content. Frontend just provides line numbers as reference points.

---

### `src-tauri/src/commands/`

Tauri command handlers that call service functions.

**`config.rs`:**
- `get_api_key()` → `config_service::get_api_key()`
- `set_api_key(key)` → `config_service::set_api_key()`

**`projects.rs`:**
- `list_projects()` → `file_service::list_all_projects()`
- `get_project(id)` → `file_service::load_project()`
- `create_project(title, desc)` → `file_service::create_new_project()`
- `delete_project(id)` → `file_service::delete_project_dir()`
- `get_page_content(...)` → `file_service::load_page_content()`
- `save_page_content(...)` → `file_service::save_page_content()`
- `add_page(...)` → `file_service::add_page_to_project()`
- `reorder_pages(...)` → Updates meta.page_order

**`ai.rs`:**
- `generate_learning(topic, depth)` → `ai_service::generate_learning_material()`
- `expand_selection(...)` → `ai_service::expand_selection_with_ai()`
- `remove_expansion(...)` → Regex removes `<details>` block from markdown

---

## Data Storage

**Location:** `~/Library/Application Support/TutorGPT/`

```
TutorGPT/
├── config.json
│   {
│     "api_key": "sk-...",
│     "theme": ""
│   }
│
└── projects/
    └── rust-basics/
        ├── meta.json
        │   {
        │     "id": "rust-basics",
        │     "title": "Rust Basics",
        │     "description": "...",
        │     "created_at": "2025-...",
        │     "updated_at": "2025-...",
        │     "page_order": ["01-introduction.md", "02-ownership.md"]
        │   }
        │
        └── pages/
            ├── 01-introduction.md
            └── 02-ownership.md
```

---

## Design System

**Current Aesthetic:** Book-inspired reading experience

**Colors:**
- Background: Warm cream paper (`#f4f1eb`)
- Elevated surfaces: Lighter cream (`#faf8f4`)
- Text: Warm dark brown-black (`#2c2416`)
- Accents: Same as text color (black buttons)

**Typography:**
- Body: Georgia, Times New Roman, Palatino (serif)
- UI: San Francisco, Helvetica Neue (sans-serif)
- Code: Courier New (monospace)
- Book features: Justified text, paragraph indentation, drop caps, small-caps headings

**Visual Effects:**
- Paper grain texture (2.5% opacity SVG noise)
- Subtle edge vignette
- Flash animation for content updates (warm tan highlight)
- No hover animations

---

## What's NOT Implemented

| Feature | Location | Status |
|---------|----------|--------|
| AI content generation | `ai_service.rs:generate_learning_material()` | Placeholder |
| AI text expansion | `ai_service.rs:expand_selection_with_ai()` | Placeholder |
| PocketFlow integration | Throughout `ai_service.rs` | Not started |
| Expansion metadata tracking | `.expansions.json` | Designed but not implemented |
| Project editing | UI | Can create/delete, not edit metadata |
| Page reordering UI | UI | Command exists, no drag-drop |
| Export (PDF/HTML) | - | Not implemented |

## What WAS Recently Updated

| Feature | Location | Changes |
|---------|----------|---------|
| Multi-line selection | `markdown.js:findTextInMarkdown()` | Now properly detects text across multiple lines (up to 20) |
| Line number passing | `reader.js:handleAskQuestion()` | Sends `{startLine, endLine, selectedText, question}` instead of full markdown replacement logic |
| Flash animation | `reader.js:flashChangedLines()` | Added 1.5s warm highlight animation for updated content |
| Book-style design | `input.css` | Complete redesign: serif fonts, justified text, warm paper colors, no hover animations |
| Reader layout | `input.css:.reader-container` | Centered 800px book page with elevated card styling |
| Loading states | `loading.js` | New component for full-screen loading overlay |

---

## Running the App

```bash
# Development (with hot reload)
npm run tauri dev

# Build for production
npm run tauri build

# Rebuild CSS after changing input.css
npx @tailwindcss/cli -i src/input.css -o src/output.css

# Check Rust compilation
cargo check --manifest-path src-tauri/Cargo.toml
```

---

## Adding Features

### New Tauri Command

1. Add function to `src-tauri/src/services/*.rs`
2. Add command wrapper in `src-tauri/src/commands/*.rs`
3. Register in `src-tauri/src/lib.rs` generate_handler
4. Add JS wrapper in `src/js/api.js`
5. Call from page component

### New Page

1. Create `src/js/pages/mypage.js` with `export async function renderMyPage()`
2. Import in `app.js`
3. Register route: `router.register('/mypage', renderMyPage)`

### New Component

1. Create `src/js/components/mycomponent.js`
2. Export initialization function
3. Import and use in page files
