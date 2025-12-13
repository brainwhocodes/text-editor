# AI Code Editor (Rust + Slint + OpenRouter) — TODO / Implementation Plan

This file is the execution checklist for building a cross-platform AI-powered code editor with a **high-end custom editor engine** (not a simple text box), a **Slint** UI, and **OpenRouter** for LLM inference.

## Guiding Constraints / Non-Functional Requirements

- [ ] Cross-platform: Windows / macOS / Linux
- [ ] Modular architecture with plugin-like extension points
- [ ] Clean separation:
  - UI (Slint)
  - Editor engine (buffer/layout/render)
  - Workspace/filesystem layer
  - AI service (OpenRouter)
- [x] Secure key handling for OpenRouter (OS keychain)

---

## 0) Repo + Crate Layout (foundation)

- [x] Create a Rust workspace with crates:
  - [x] `app` (Slint UI shell + wiring)
  - [x] `editor_core` (domain models, event bus, commands)
  - [x] `workspace` (project tree, fs ops, file watching)
  - [x] `editor` (buffer/layout/render/input)
  - [x] `syntax` (tree-sitter integration, tokenization)
  - [x] `ai` (OpenRouter client + streaming + prompt/context)
  - [x] `diff` (diff view model + patch apply/reject)
  - [x] `plugins` (plugin API + host scaffolding)

- [x] Establish a strict “UI thread vs background runtime” rule:
  - UI thread: Slint rendering + user input dispatch
  - Background: async services (tokio)

- [x] Define shared types in `core`:
  - [x] `Command` (UI intents)
  - [x] `Event` (service->UI updates)
  - [x] `AppState` slices (documents, workspace, chat, theme)
  - [x] `Result` / error taxonomy

---

## 1) High-End Editor Engine (core differentiator)

### 1.1 Document model + buffer
- [x] Implement text storage using `ropey` (rope buffer)
  - [x] Efficient insert/delete
  - [x] Line indexing utilities (line -> byte/char offset)
  - [x] Snapshots/versions for concurrency + diff

- [x] Implement selections/cursors
  - [x] Basic cursor + selection types
  - [x] Multi-cursor support (primary + secondary carets)
  - [ ] Rectangular selection (optional later)

- [x] Undo/redo
  - [x] Transaction model (group edits)
  - [x] Coalescing for typing
  - [x] Undo integrates with multi-cursor edits

### 1.2 Layout + rendering pipeline
- [ ] Text shaping + font metrics
  - [ ] Choose a shaping stack:
    - [ ] `cosmic-text` (recommended) or
    - [ ] `swash` + shaping glue

- [x] Incremental line layout cache
  - [x] Recompute only affected lines on edits
  - [x] Support soft wrap (toggleable)

- [x] Viewport rendering
  - [x] Render only visible lines
  - [x] Fast scrolling with caching

- [x] Editor visuals
  - [x] Gutter (line numbers)
  - [x] Cursor + selection rendering
  - [x] Current line highlight
  - [ ] Whitespace rendering toggles (optional)

### 1.3 Input & editing commands
- [x] Keybindings system (configurable)
- [ ] Core commands:
  - [x] insert text / newline
  - [x] backspace/delete word/line
  - [x] move cursor (by char/word/line)
  - [x] copy/cut/paste
  - [x] indent/outdent
  - [x] duplicate line
  - [x] comment toggle (language-aware later)

### 1.4 Search
- [x] Find (incremental)
- [x] Replace (single + replace-all)
- [ ] Optional: regex search later

---

## 2) Syntax Highlighting + Language Features

### 2.1 Tree-sitter integration
- [ ] `syntax` crate with:
  - [ ] language registry (by file extension)
  - [ ] incremental parse on edits (tree-sitter)
  - [ ] capture highlights using query files

- [ ] Highlight token stream
  - [ ] Produce per-line spans (start..end -> token type)
  - [ ] Incremental update for only invalidated regions

### 2.2 Optional IDE features (later phases)
- [ ] Diagnostics/linters integration hooks
- [ ] Symbol outline (tree-sitter queries)
- [ ] Go-to-definition (pluggable)

---

## 3) Slint UI (Editor + Panels)

### 3.1 Layout
- [ ] Split-pane layout (MVP fixed):
  - Left: Explorer
  - Center: Editor tabs + editor view
  - Right/Bottom: Chat + Diff (tabbed)

- [ ] Later: docking layout manager (drag/drop panels)

### 3.2 UI components
- [ ] ExplorerPanel: workspace tree + file ops
- [ ] Tabs: open documents, dirty indicator, close
- [ ] EditorWidget: custom rendering surface bound to `editor` engine
- [ ] ChatPanel: convo list, streaming output, context toggles
- [ ] DiffPanel: preview AI edits, apply/reject
- [ ] StatusBar: cursor pos, language mode, model name, git info (later)
- [ ] CommandPalette: fuzzy command runner (later)

### 3.3 UI-thread integration
- [ ] Event bridge: background services emit events; UI applies them via Slint event loop
- [ ] Performance: throttle high-frequency editor repaint events

---

## 4) Workspace + File Operations

- [ ] Open folder as workspace
- [ ] Tree building + caching
- [ ] File ops: create/rename/delete files/folders
- [ ] File watching (`notify`) to refresh explorer and detect external changes
- [ ] Per-workspace settings (recent files, last open tabs)

---

## 5) Diff Viewer + Patch Apply/Reject

### 5.1 Diff generation
- [ ] Compute diffs between:
  - current document buffer vs proposed buffer, or
  - apply unified diff to buffer snapshot and show resulting diff

### 5.2 Patch format contract (AI)
- [ ] Prefer structured edit protocol for reliability:
  - Option A: unified diff with file path headers
  - Option B (recommended for safety): JSON edits:
    - file path
    - base version/hash
    - list of operations (range replace)

### 5.3 Apply/reject
- [ ] Validate applies cleanly against expected base version
- [ ] Apply into editor buffer (not directly to disk)
- [ ] Integrate with undo stack as a single transaction
- [ ] Conflict UI: show why apply failed + offer rebase/retry

---

## 6) OpenRouter AI Integration

### 6.1 Secure API key handling
- [ ] Store key in OS keychain (`keyring`)
- [ ] Settings UI to set/remove key
- [ ] Never persist key in plaintext config

### 6.2 Chat + streaming
- [ ] OpenRouter client (reqwest)
- [ ] Streaming output into ChatPanel
- [ ] Conversation persistence (local JSON/SQLite)

### 6.3 Context management
- [ ] Context providers:
  - active file contents
  - selection(s)
  - diagnostics/symbol outline (later)
  - user-picked files

- [ ] Token budget enforcement:
  - summarize older turns
  - truncate large files with line ranges

### 6.4 Inline completions / suggestions
- [ ] “ghost text” suggestion rendering at caret
- [ ] Accept / reject / next suggestion
- [ ] Debounced request policy (avoid spamming)

---

## 7) Theming

- [ ] Theme model with semantic tokens (not raw colors everywhere)
- [ ] Load themes from JSON or YAML
- [ ] Apply theme across:
  - editor (tokens + UI chrome)
  - chat panel
  - diff view

---

## 8) Plugin-like Architecture

### 8.1 Extension points
- [ ] Command registration (command palette)
- [ ] Context providers (add info to prompts)
- [ ] Editor actions (format, refactor)
- [ ] Model tools (structured edit emitters)

### 8.2 Plugin hosting model
- [ ] Prefer out-of-process plugins (JSON-RPC) for safety/cross-platform
- [ ] Capability manifest per plugin (filesystem access, network, etc.)

---

## Development Phases (recommended order)

### Phase A — Skeleton + documents
- [ ] App boots, workspace opens
- [ ] Tabs + open/save
- [ ] Custom editor widget renders text and supports basic editing

### Phase B — High-end editor hardening
- [ ] Rope-based buffer + undo/redo
- [ ] Multi-cursor
- [ ] Search/replace
- [ ] Performance work (viewport render, caches)

### Phase C — Syntax highlighting
- [ ] Tree-sitter incremental parse
- [ ] Highlight spans integrated into renderer

### Phase D — AI + Diff pipeline
- [ ] OpenRouter chat + streaming
- [ ] Context management
- [ ] Patch proposal -> diff preview -> apply/reject

### Phase E — Themes + polish
- [ ] Theme loader + editor tokens
- [ ] Keybinding config
- [ ] Command palette

### Phase F — Plugins
- [ ] JSON-RPC plugin host
- [ ] First sample plugin(s)

---

## Acceptance Criteria (MVP-ish but “high-end editor”)

- [ ] Smooth editing on large files (target: 10+ MB without obvious lag)
- [ ] Incremental syntax highlighting via tree-sitter
- [ ] Multi-tab editing, undo/redo, search/replace
- [ ] OpenRouter chat with code-aware context
- [ ] AI suggested edits previewable in diff viewer and safely applicable as a single undo step
- [ ] Themes apply consistently across editor/chat/diff
