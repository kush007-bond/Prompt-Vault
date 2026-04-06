# PromptVault — Product & Technical Requirements Document

> **Version:** 2.0.0
> **Date:** March 2026
> **Status:** Active — Revised for Local-First Architecture
> **License:** MIT (planned)
> **Repository:** github.com/your-org/promptvault *(placeholder)*

---

## Table of Contents

**Part I — Product Requirements Document**
1. [Vision & Mission](#1-vision--mission)
2. [Problem Statement](#2-problem-statement)
3. [Core Philosophy](#3-core-philosophy)
4. [Target Users & Personas](#4-target-users--personas)
5. [Feature List](#5-feature-list)
6. [User Stories](#6-user-stories)
7. [Non-Functional Requirements](#7-non-functional-requirements)
8. [What We Deliberately Excluded](#8-what-we-deliberately-excluded)

**Part II — Technical Requirements Document**
9. [Architecture Overview](#9-architecture-overview)
10. [Technology Stack](#10-technology-stack)
11. [Local Storage Design](#11-local-storage-design)
12. [AI Provider Integrations](#12-ai-provider-integrations)
13. [CLI Tool Integrations](#13-cli-tool-integrations)
14. [Security Architecture](#14-security-architecture)
15. [Plugin & Adapter System](#15-plugin--adapter-system)
16. [Data Portability & Sync](#16-data-portability--sync)
17. [Project Structure](#17-project-structure)
18. [Risks & Mitigations](#18-risks--mitigations)
19. [Roadmap & Milestones](#19-roadmap--milestones)

---

# Part I — Product Requirements Document

---

## 1. Vision & Mission

**Vision**
A world where every developer and knowledge worker owns their AI interactions completely — where prompts are a personal, private asset stored on their own machine, accessible offline, and usable across every AI tool they touch.

**Mission**
To build the most capable, privacy-respecting, open-source prompt manager that works entirely on the user's device — connecting directly to any AI model and any coding CLI without a server, without accounts, and without ever asking for trust.

**One-line pitch**
*Your prompts. Your machine. Your keys. Zero middlemen.*

---

## 2. Problem Statement

AI power users — developers especially — are hitting a wall that no existing tool addresses:

**Prompt loss** — Carefully engineered prompts live inside chat windows that scroll into oblivion. There is no native save, no search, no history across tools.

**Fragmentation** — A developer might use Claude Code in the terminal, ChatGPT in the browser, and Cursor in the editor — all with different prompts, no shared library, no way to reuse work across them.

**Vendor lock-in** — Existing prompt managers are cloud-first SaaS products. They hold your prompts on their servers, require accounts, and could disappear, pivot, or change pricing at any time.

**Privacy** — System prompts, workflow prompts, and project context often contain sensitive IP. Uploading these to a third-party service is a security and legal risk many teams cannot accept.

**No CLI bridge** — There is no tool that lets you manage prompts in a GUI and inject them directly into Claude Code, Aider, Cursor, or Continue.dev. Developers copy-paste manually between tools every single day.

---

## 3. Core Philosophy

These are the constraints that define every decision in this project. They are not features — they are principles. Any proposed addition that violates them is rejected.

### 3.1 Local by Default
All user data — prompts, collections, tags, version history, settings, API keys — lives on the user's device. Nothing is transmitted to any PromptVault-controlled server. Ever.

### 3.2 Zero Account Requirement
The application is fully functional without creating an account, without providing an email address, and without an internet connection. A user can install and use the full product completely offline using local AI models.

### 3.3 Open Source & Auditable
Every line of code is public on GitHub under the MIT license. Users can inspect exactly what the application does. No telemetry, no crash reporting, no analytics without explicit opt-in with full transparency about what is collected and where it goes.

### 3.4 Direct AI Connections Only
When a user runs a prompt against an AI provider, the request travels directly from their machine to the provider's API using their own API key. PromptVault is never a proxy, never a middleman, and never touches the API key beyond reading it from the local OS keychain to construct the request.

### 3.5 No Vendor Lock-in
Prompts are stored in SQLite — a universally readable, open format. Users can export everything to Markdown, JSON, or CSV at any time. The database file itself can be opened with any SQLite browser. The user's data is never held hostage.

### 3.6 Future-Proof by Design
Every external integration — AI providers, CLI tools, export formats — is implemented as a pluggable adapter behind a stable interface. Adding a new AI provider or a new CLI tool requires writing one adapter file, not modifying core. The community can ship adapters independently.

---

## 4. Target Users & Personas

### Persona A — The Solo Developer

| Attribute | Detail |
|-----------|--------|
| **Name** | Rohan Mehta |
| **Role** | Freelance full-stack developer |
| **Daily Tools** | Claude Code, VS Code, Cursor, ChatGPT |
| **Pain Points** | Rebuilds the same system prompts for every project; can't inject saved prompts into Claude Code sessions quickly |
| **Goals** | One keyboard shortcut to search and inject any saved prompt into whatever CLI tool he's currently using |
| **Local requirement** | Works on client projects — cannot upload prompts containing client IP to any cloud |

### Persona B — The AI-Native Power User

| Attribute | Detail |
|-----------|--------|
| **Name** | Aisha Patel |
| **Role** | AI researcher and content strategist |
| **Daily Tools** | ChatGPT, Claude.ai, Gemini, Ollama |
| **Pain Points** | Has 200+ prompts scattered across 4 browser tabs; wants to test the same prompt across different models |
| **Goals** | Organized library, one-click cross-model testing, full version history |
| **Local requirement** | Runs Ollama locally; wants zero internet dependency for her core workflow |

### Persona C — The Privacy-Conscious Enterprise Developer

| Attribute | Detail |
|-----------|--------|
| **Name** | Marcus Kim |
| **Role** | Senior engineer at a fintech company |
| **Daily Tools** | Aider, Continue.dev, internal GPT-4 deployment |
| **Pain Points** | Company policy forbids uploading code context to third-party SaaS; needs a prompt manager that legal will approve |
| **Goals** | Fully local tool, auditable source code, exportable data, no cloud footprint |
| **Local requirement** | Mandatory — would not use the product if it had any cloud dependency |

---

## 5. Feature List

Priority levels: **P0** = MVP (must ship), **P1** = v1.1, **P2** = v2.0, **P3** = future consideration.

---

### 5.1 Core Prompt Management

| ID | Feature | Priority |
|----|---------|----------|
| F-001 | Create, read, update, delete (CRUD) prompts with rich Markdown editor | P0 |
| F-002 | Syntax highlighting inside prompt body (code blocks, variables) | P0 |
| F-003 | One-click copy to clipboard | P0 |
| F-004 | Collections and nested folders | P0 |
| F-005 | Tagging system with multi-tag filtering | P0 |
| F-006 | Full-text search using SQLite FTS5 (sub-50ms, offline) | P0 |
| F-007 | Complete version history with side-by-side diff view | P0 |
| F-008 | Restore any previous version | P0 |
| F-009 | Pin / star prompts | P0 |
| F-010 | Recently used prompts list | P0 |
| F-011 | Duplicate a prompt | P0 |
| F-012 | Undo / redo in editor (50 steps) | P0 |
| F-013 | Character, word, and estimated token count in editor | P0 |
| F-014 | Variable interpolation with `{variable_name}` syntax | P0 |
| F-015 | Variable type hints and default values | P0 |
| F-016 | Variable fill form — modal to populate variables before sending | P0 |
| F-017 | Command palette (Cmd/Ctrl + K) with fuzzy search | P0 |
| F-018 | Dark / light / system theme | P0 |
| F-019 | Bulk select and bulk actions (delete, move, tag) | P1 |
| F-020 | Import from JSON, Markdown, CSV, plain text | P1 |
| F-021 | Export all prompts (JSON, Markdown bundle, CSV) | P1 |
| F-022 | Drag-and-drop prompt reordering within collections | P1 |
| F-023 | Color labels on collections | P1 |
| F-024 | Smart collections — auto-populate by tag, model target, or date | P1 |
| F-025 | Focus / distraction-free writing mode | P1 |
| F-026 | Regex-powered search | P2 |
| F-027 | Prompt templates starter library (built-in, offline) | P2 |
| F-028 | Local semantic search using on-device embeddings | P2 |

---

### 5.2 AI Provider Integrations

| ID | Feature | Priority |
|----|---------|----------|
| F-029 | OpenAI direct integration (user's own API key) | P0 |
| F-030 | Anthropic Claude direct integration (user's own API key) | P0 |
| F-031 | Ollama integration — run prompts against local models | P0 |
| F-032 | Google Gemini direct integration | P0 |
| F-033 | Mistral direct integration | P1 |
| F-034 | Any OpenAI-compatible endpoint (LM Studio, Jan, LocalAI, custom) | P0 |
| F-035 | Model selector — pick which model to run a prompt against | P0 |
| F-036 | Run prompt inline and see response in app | P0 |
| F-037 | Response history — last N responses per prompt, stored locally | P1 |
| F-038 | Compare responses from two different models side by side | P1 |
| F-039 | Streaming response display | P1 |
| F-040 | Token usage and estimated cost display (per provider) | P1 |
| F-041 | API key management UI — add, edit, test, remove keys per provider | P0 |
| F-042 | API keys stored in OS keychain, never in SQLite | P0 |
| F-043 | Connection health check per provider | P0 |
| F-044 | Offline mode indicator — shows which providers are unavailable | P0 |

---

### 5.3 CLI Tool Integrations

| ID | Feature | Priority |
|----|---------|----------|
| F-045 | Claude Code — inject prompt into `CLAUDE.md` (project or global) | P0 |
| F-046 | Cursor — inject prompt into `.cursorrules` for selected project | P0 |
| F-047 | Continue.dev — inject prompt into `config.json` system prompt | P0 |
| F-048 | Aider — inject via `--system-prompt` flag or `.aider.conf.yml` | P0 |
| F-049 | GitHub Copilot CLI — shell alias and stdin pipe generation | P1 |
| F-050 | Project directory picker — point injection at any project folder | P0 |
| F-051 | Dry-run preview — show exactly what will be written before writing | P0 |
| F-052 | Injection history log — what was injected where and when | P1 |
| F-053 | Multi-prompt concat — combine multiple prompts for injection | P1 |
| F-054 | Custom CLI adapter — user-defined injection target via plugin | P2 |
| F-055 | Shell snippet generator — wrap prompt as reusable shell alias | P2 |

---

### 5.4 Security & Privacy

| ID | Feature | Priority |
|----|---------|----------|
| F-056 | All data stored in local SQLite file only | P0 |
| F-057 | API keys stored exclusively in OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service) | P0 |
| F-058 | Optional AES-256 encryption of the local SQLite database with a user passphrase | P1 |
| F-059 | App lock — require passphrase or biometric on launch | P1 |
| F-060 | Zero telemetry by default — no data leaves device without explicit opt-in | P0 |
| F-061 | Opt-in anonymous crash reporting (no prompt data included) | P2 |
| F-062 | Open source license (MIT) with public build reproducibility | P0 |
| F-063 | No auto-update that executes remote code — update notifications only | P0 |

---

### 5.5 Data Portability & Backup

| ID | Feature | Priority |
|----|---------|----------|
| F-064 | Manual export of full library to a single JSON file | P0 |
| F-065 | Manual export to Markdown bundle (one file per prompt) | P0 |
| F-066 | Import from exported JSON — full restore | P0 |
| F-067 | One-click backup SQLite database file to any local path | P1 |
| F-068 | Scheduled automatic local backup (configurable interval, local path only) | P1 |
| F-069 | Opt-in sync via Dropbox / iCloud / Google Drive (user configures path — app just writes a file) | P2 |
| F-070 | Export single prompt as GitHub Gist (uses user's own GitHub token) | P2 |
| F-071 | Import from GitHub Gist URL | P2 |

---

### 5.6 Developer & Power User Features

| ID | Feature | Priority |
|----|---------|----------|
| F-072 | Full keyboard navigation — every action reachable without mouse | P0 |
| F-073 | Configurable keyboard shortcuts | P1 |
| F-074 | Local REST API server (optional, localhost only) — expose prompt library to other tools | P1 |
| F-075 | CLI companion (`promptvault get <name>`, `promptvault list`, `promptvault run`) | P1 |
| F-076 | MCP server mode — expose prompts as tools via Model Context Protocol | P2 |
| F-077 | Plugin system — third-party adapters for new AI providers and CLI tools | P2 |
| F-078 | Scriptable prompt chaining (run prompt A, pipe output into prompt B) | P2 |
| F-079 | Prompt quality checklist (length, variable completeness, clarity score — local heuristics) | P2 |

---

## 6. User Stories

### MVP User Stories (P0)

**US-001 — First launch, no internet**
*As a developer installing PromptVault for the first time, I can open the app, create a collection, and save my first prompt without an internet connection, without creating an account, and without seeing any login screen.*

**US-002 — CLI injection: Claude Code**
*As a developer using Claude Code, I can search my prompt library from PromptVault, select a system prompt, choose my project directory, and have it written to `CLAUDE.md` in one action — so my next `claude` session picks it up automatically.*

**US-003 — Direct AI run**
*As a user, I can open a prompt, click "Run", select a provider (e.g. Anthropic Claude), fill in any variables via a pop-up form, and see the streamed response inside the app — using my own API key, with no PromptVault server involved.*

**US-004 — Offline with Ollama**
*As a privacy-conscious developer, I can run any saved prompt against a locally-running Ollama model with no internet connection whatsoever — not even for the AI call.*

**US-005 — Version restore**
*As a user, I can view the full edit history of any prompt, see a character-level diff between any two versions, and restore any previous version without losing the current one.*

**US-006 — API key security**
*As a user, I can add my OpenAI API key once and trust that it is stored in my OS keychain — not in a database file, not in a config file, not anywhere that could be accidentally committed to git or read by another application.*

**US-007 — Full data export**
*As a user, I can export my entire prompt library as a single portable JSON file at any time, knowing I can import it into any future version of PromptVault or read it with any text editor.*

---

## 7. Non-Functional Requirements

| Category | Requirement |
|----------|-------------|
| **Performance** | App launch < 1s on modern hardware; search results < 50ms on a library of 10,000 prompts; prompt CRUD < 10ms |
| **Offline** | 100% of core features (CRUD, search, version history, CLI injection) work with no internet connection |
| **Binary size** | App bundle < 20 MB (Tauri target); no Electron bloat |
| **Memory** | Idle RAM usage < 80 MB; active usage < 200 MB |
| **Storage** | SQLite database < 50 MB for 50,000 prompts (text-only, no embeddings) |
| **Platform** | macOS 12+, Windows 10+, Linux (Ubuntu 20.04+, Fedora 36+) — all from a single codebase |
| **Accessibility** | WCAG 2.1 AA — full keyboard navigation, screen reader compatible, scalable font size |
| **Reliability** | SQLite WAL mode — no data loss on crash or forced quit; atomic writes only |
| **Updates** | Notification-only auto-update — user initiates download and install; no silent code execution |
| **Reproducibility** | GitHub Actions CI builds produce reproducible binaries; users can build from source |

---

## 8. What We Deliberately Excluded

These are conscious, non-negotiable exclusions. They are not on any roadmap.

| Excluded Feature | Reason |
|-----------------|--------|
| Cloud storage for prompts | Violates local-first principle; third party holds user data |
| User accounts / sign-up | Unnecessary for a local tool; creates privacy risk |
| PromptVault-managed sync server | Would make us a cloud provider; contradicts open-source trust model |
| Third-party analytics (Mixpanel, Amplitude, PostHog cloud) | No telemetry without explicit user opt-in |
| Browser extension that sends data to our servers | All browser extension features must work via localhost API only |
| Team workspaces with cloud backend | Export/import is the sharing mechanism; no cloud collab layer |
| Monetization / paywalled features | MIT license; 100% of features free and open |
| AI features that proxy through PromptVault's own API key | Users always use their own keys; we have no key management server |

---

# Part II — Technical Requirements Document

---

## 9. Architecture Overview

PromptVault is a **local-first desktop application** built with Tauri v2. The architecture has three distinct layers with a strict dependency rule: the UI layer cannot talk to the OS or filesystem directly; all privileged operations go through the Rust core.

```
┌─────────────────────────────────────────────────────┐
│                   UI Layer (React)                   │
│   Vite + React 19 + TypeScript + Tailwind CSS        │
│   Renders in a WebView — no DOM access to Rust      │
└──────────────────┬──────────────────────────────────┘
                   │  Tauri IPC (invoke / events)
┌──────────────────▼──────────────────────────────────┐
│                  Rust Core (Tauri v2)                │
│   Commands · State · File system · OS keychain       │
│   SQLite via rusqlite · HTTP client (reqwest)        │
└───┬─────────────┬──────────────┬────────────────────┘
    │             │              │
┌───▼───┐   ┌────▼────┐   ┌────▼────────────┐
│SQLite │   │OS       │   │AI Providers     │
│.db    │   │Keychain │   │(direct HTTP)    │
│local  │   │(keys)   │   │OpenAI/Anthropic │
└───────┘   └─────────┘   │Ollama/Gemini    │
                           └─────────────────┘
```

**Key architectural decisions:**

- The Rust core owns all I/O — file system, SQLite, keychain, and outbound HTTP. The React layer is a pure UI.
- All communication between React and Rust uses Tauri's typed IPC (`invoke`). No raw `eval`, no arbitrary shell execution from the frontend.
- AI API calls are made from Rust using `reqwest` — not from the browser WebView. This ensures API keys never touch the JS layer.
- SQLite runs in WAL (Write-Ahead Logging) mode for crash safety and concurrent reads.

---

## 10. Technology Stack

### 10.1 Desktop Shell

| Layer | Technology | Reason |
|-------|-----------|--------|
| Desktop framework | **Tauri v2** | Rust-based, 5–15 MB binaries vs Electron's 150+ MB; proper OS security sandbox; native OS APIs |
| Frontend runtime | **WebView** (WKWebView on macOS, WebView2 on Windows, WebKitGTK on Linux) | Provided by Tauri; no bundled Chromium |
| Build tool | **Vite 6** | Sub-second HMR; native ESM; fastest dev experience |

### 10.2 Frontend

| Layer | Technology | Reason |
|-------|-----------|--------|
| Framework | **React 19** | Mature ecosystem; concurrent features; wide talent pool |
| Language | **TypeScript 5.5 (strict)** | End-to-end type safety with generated Tauri command types |
| Styling | **Tailwind CSS 4** | Utility-first; zero runtime; tree-shaken to < 10 KB in prod |
| Component primitives | **Radix UI** | Accessible, headless; fully keyboard-navigable |
| Editor | **CodeMirror 6** | Lightweight; extensible; handles Markdown + variable highlighting |
| State management | **Zustand** | Minimal; no boilerplate; works perfectly in a local-only app |
| Data fetching (Tauri IPC) | **TanStack Query** | Caching, loading states, and refetch logic for Tauri command calls |
| Icons | **Lucide React** | MIT; tree-shakable; consistent |
| Animations | **Framer Motion** | Declarative; performant; respects `prefers-reduced-motion` |

### 10.3 Rust Core

| Layer | Technology | Reason |
|-------|-----------|--------|
| Desktop framework | **Tauri v2** | Provides the command system, event bus, and OS API bridges |
| Database | **rusqlite** | Direct SQLite bindings for Rust; bundled SQLite, no external dependency |
| ORM / query builder | **Diesel** (or raw rusqlite) | Type-safe queries; migration runner |
| HTTP client | **reqwest** (async, TLS) | Making direct AI API calls from Rust |
| Async runtime | **Tokio** | Tauri's built-in async runtime; handles concurrent AI requests |
| Serialization | **serde + serde_json** | Tauri IPC payload serialization |
| OS keychain | **keyring crate** | Cross-platform keychain abstraction (macOS/Win/Linux) |
| Encryption | **AES-GCM via RustCrypto** | Optional SQLite DB encryption |
| CLI companion | **clap** | Argument parsing for the `promptvault` CLI binary |

### 10.4 Local AI (Ollama)

Ollama runs as a separate local process on the user's machine. PromptVault communicates with it via `localhost:11434` using Ollama's OpenAI-compatible REST API. No special library needed — it uses the same HTTP client as cloud providers.

### 10.5 Tooling

| Tool | Purpose |
|------|---------|
| **pnpm** | Fast, disk-efficient package manager |
| **Vitest** | Unit and integration tests for TypeScript |
| **Playwright** | End-to-end tests (Tauri supports Playwright via WebDriver) |
| **cargo test** | Rust unit and integration tests |
| **GitHub Actions** | CI/CD: lint → test → build (macOS, Windows, Linux) → sign → release |
| **Tauri Updater** | Notification-only update checks; user-initiated download |

---

## 11. Local Storage Design

### 11.1 Database Location

The SQLite database file lives at the OS-appropriate app data path, managed by Tauri:

```
macOS:   ~/Library/Application Support/com.promptvault.app/promptvault.db
Windows: C:\Users\<user>\AppData\Roaming\com.promptvault.app\promptvault.db
Linux:   ~/.local/share/com.promptvault.app/promptvault.db
```

Users can see this path in Settings > Storage. They can move the database file and point the app at the new location.

### 11.2 SQLite Configuration

```sql
PRAGMA journal_mode = WAL;       -- Crash-safe, allows concurrent reads
PRAGMA foreign_keys = ON;        -- Enforce referential integrity
PRAGMA synchronous = NORMAL;     -- Good durability/performance balance
PRAGMA cache_size = -32000;      -- 32 MB page cache
PRAGMA temp_store = MEMORY;      -- Temp tables in RAM
PRAGMA mmap_size = 268435456;    -- 256 MB memory-mapped I/O
```

### 11.3 Schema

#### prompts

```sql
CREATE TABLE prompts (
  id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
  title           TEXT NOT NULL,
  body            TEXT NOT NULL,
  model_target    TEXT,                          -- e.g. 'gpt-4o', 'claude-3-7-sonnet', null = any
  collection_id   TEXT REFERENCES collections(id) ON DELETE SET NULL,
  is_pinned       INTEGER NOT NULL DEFAULT 0,
  is_archived     INTEGER NOT NULL DEFAULT 0,
  use_count       INTEGER NOT NULL DEFAULT 0,
  sort_order      REAL    NOT NULL DEFAULT 0,    -- fractional indexing
  created_at      TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Full-text search index (SQLite FTS5)
CREATE VIRTUAL TABLE prompts_fts USING fts5(
  title, body, tags,
  content='prompts',
  content_rowid='rowid',
  tokenize='unicode61 remove_diacritics 2'
);
```

#### prompt_versions

```sql
CREATE TABLE prompt_versions (
  id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
  prompt_id       TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
  title_snapshot  TEXT NOT NULL,
  body_snapshot   TEXT NOT NULL,
  changed_at      TEXT NOT NULL DEFAULT (datetime('now')),
  change_note     TEXT
);
```

#### variables

```sql
CREATE TABLE variables (
  id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
  prompt_id       TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
  name            TEXT NOT NULL,                -- matches {variable_name} in body
  display_label   TEXT,
  type            TEXT NOT NULL DEFAULT 'text', -- text | multiline | select | number
  default_value   TEXT,
  options         TEXT,                         -- JSON array for select type
  required        INTEGER NOT NULL DEFAULT 1,
  sort_order      INTEGER NOT NULL DEFAULT 0
);
```

#### collections

```sql
CREATE TABLE collections (
  id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
  name            TEXT NOT NULL,
  parent_id       TEXT REFERENCES collections(id) ON DELETE CASCADE,
  color           TEXT,                         -- hex color string
  icon            TEXT,                         -- icon name from Lucide
  is_smart        INTEGER NOT NULL DEFAULT 0,
  smart_filter    TEXT,                         -- JSON filter definition
  sort_order      REAL    NOT NULL DEFAULT 0,
  created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### tags

```sql
CREATE TABLE tags (
  id    TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
  name  TEXT NOT NULL UNIQUE COLLATE NOCASE,
  color TEXT
);

CREATE TABLE prompt_tags (
  prompt_id TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
  tag_id    TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY (prompt_id, tag_id)
);
```

#### response_history

```sql
CREATE TABLE response_history (
  id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
  prompt_id       TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
  provider        TEXT NOT NULL,                -- 'openai' | 'anthropic' | 'ollama' | ...
  model           TEXT NOT NULL,
  input_snapshot  TEXT NOT NULL,               -- resolved prompt (variables filled)
  response        TEXT NOT NULL,
  tokens_input    INTEGER,
  tokens_output   INTEGER,
  duration_ms     INTEGER,
  ran_at          TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### injection_log

```sql
CREATE TABLE injection_log (
  id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
  prompt_id       TEXT REFERENCES prompts(id) ON DELETE SET NULL,
  cli_target      TEXT NOT NULL,               -- 'claude_code' | 'cursor' | 'aider' | ...
  project_path    TEXT NOT NULL,
  injected_content TEXT NOT NULL,              -- exact string written
  injected_at     TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### settings

```sql
CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
-- Stores: theme, default_provider, default_model, editor_font_size,
--         backup_interval, backup_path, locale, etc.
-- API keys are NEVER stored here — they live in the OS keychain.
```

### 11.4 Migrations

Schema migrations are managed using embedded SQL migration files, applied sequentially at app startup via the Rust core. Migration state tracked in a `schema_migrations` table. Migrations are additive-only — no destructive migrations.

```
src-tauri/
  migrations/
    0001_initial_schema.sql
    0002_add_injection_log.sql
    0003_add_response_history.sql
    ...
```

---

## 12. AI Provider Integrations

### 12.1 Provider Adapter Interface (Rust)

Every AI provider implements the `ProviderAdapter` trait. Adding a new provider = writing one struct that implements this trait.

```rust
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn requires_api_key(&self) -> bool;
    fn supported_models(&self) -> Vec<ModelInfo>;

    async fn run_prompt(
        &self,
        prompt: &str,
        model: &str,
        options: RunOptions,
    ) -> Result<ProviderResponse, AdapterError>;

    async fn stream_prompt(
        &self,
        prompt: &str,
        model: &str,
        options: RunOptions,
        tx: mpsc::Sender<StreamChunk>,
    ) -> Result<(), AdapterError>;

    async fn health_check(&self) -> Result<bool, AdapterError>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AdapterError>;
}
```

### 12.2 Provider Implementations

#### OpenAI

```
Endpoint:    https://api.openai.com/v1/chat/completions
Auth:        Bearer token from OS keychain key "promptvault.openai"
Models:      gpt-4o, gpt-4o-mini, o1, o1-mini, o3 (fetched via /v1/models)
Streaming:   Server-Sent Events → Tauri event → React
```

#### Anthropic Claude

```
Endpoint:    https://api.anthropic.com/v1/messages
Auth:        x-api-key header from OS keychain key "promptvault.anthropic"
Models:      claude-opus-4-6, claude-sonnet-4-6, claude-haiku-4-5 (config file)
Streaming:   SSE stream → Tauri event → React
```

#### Ollama (local)

```
Endpoint:    http://localhost:11434/v1/chat/completions  (OpenAI-compatible)
Auth:        None required
Models:      Dynamically fetched from localhost:11434/api/tags
Streaming:   Chunked JSON → Tauri event → React
Discovery:   App polls :11434 every 30s; shows "Ollama offline" if unreachable
```

#### Gemini

```
Endpoint:    https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent
Auth:        API key query param from OS keychain key "promptvault.gemini"
Models:      gemini-2.0-flash, gemini-2.0-pro (config file)
```

#### OpenAI-Compatible (Custom)

```
Endpoint:    User-configured base URL (e.g. http://localhost:1234/v1)
Auth:        Optional Bearer token
Use cases:   LM Studio, Jan, LocalAI, vLLM, any self-hosted model
```

### 12.3 API Key Flow

```
User enters key in Settings UI
        ↓
React calls Tauri command: store_api_key(provider, key)
        ↓
Rust core calls keyring crate: keyring::Entry::new("promptvault", provider).set_password(key)
        ↓
Key stored in OS keychain — never touches SQLite, never in JS memory after this call

When running a prompt:
Rust core calls keyring::Entry::new("promptvault", provider).get_password()
        ↓
Key retrieved in Rust — builds HTTP request with Authorization header
        ↓
HTTP request goes directly to provider endpoint (no JS, no proxy)
        ↓
Response returned to React via Tauri event
```

---

## 13. CLI Tool Integrations

### 13.1 CLI Adapter Interface (Rust)

```rust
pub trait CliAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn display_name(&self) -> &str;

    // Return the target file path for a given project directory
    fn target_path(&self, project_dir: &Path) -> PathBuf;

    // Preview: return what would be written without writing
    fn preview(&self, prompt_content: &str, project_dir: &Path)
        -> Result<InjectionPreview, AdapterError>;

    // Execute: write to the target file
    fn inject(&self, prompt_content: &str, project_dir: &Path)
        -> Result<InjectionResult, AdapterError>;

    // Check if target file exists and is writable
    fn health_check(&self, project_dir: &Path) -> Result<CliStatus, AdapterError>;
}
```

### 13.2 CLI Adapter Implementations

#### Claude Code

```
Target file:  {project_dir}/CLAUDE.md  (project-level)
              ~/.claude/CLAUDE.md       (global, if "Apply globally" is selected)
Strategy:     Prepend a clearly delimited PromptVault section at top of file.
              Does not overwrite existing user content below the delimiter.
Delimiter:    <!-- promptvault:start --> ... <!-- promptvault:end -->
Idempotent:   Yes — re-injecting replaces only the delimited block.
```

#### Cursor

```
Target file:  {project_dir}/.cursorrules
Strategy:     Write full content (file is typically owned by AI rules).
              On re-inject: replaces promptvault-delimited section.
```

#### Continue.dev

```
Target file:  ~/.continue/config.json
Strategy:     Parse JSON, update the systemMessage field under the
              matching model config. Write back formatted JSON.
              Non-destructive to other config fields.
```

#### Aider

```
Target file:  {project_dir}/.aider.conf.yml  or  ~/.aider.conf.yml
Strategy:     Set system_prompt key in YAML. If key exists, replace value.
              Uses a YAML parser — does not corrupt other config.
```

#### GitHub Copilot CLI

```
Strategy:     Generate a shell function + alias that echoes the prompt
              and pipes it to `gh copilot suggest` or `gh copilot explain`.
              User copies the shell snippet to their .zshrc / .bashrc.
              No file write — just clipboard copy of the snippet.
```

### 13.3 Injection Safety Rules

- Always create a backup of the target file before writing (`{file}.promptvault.bak`).
- Always show a dry-run preview before writing — user must confirm.
- Never delete content outside the `<!-- promptvault:start/end -->` delimiters.
- Log every injection to the `injection_log` table with full content snapshot.

---

## 14. Security Architecture

### 14.1 API Key Storage

API keys are stored exclusively via the `keyring` crate which uses:

| Platform | Storage |
|----------|---------|
| macOS | Keychain (Secure Enclave on Apple Silicon) |
| Windows | Windows Credential Manager (DPAPI-encrypted) |
| Linux | Secret Service API (GNOME Keyring or KWallet) |

Keys are retrieved in Rust, used to build HTTP headers, and are never passed to the JavaScript layer.

### 14.2 Tauri Security Configuration

```toml
# tauri.conf.json — strict CSP
[security]
csp = "default-src 'self'; script-src 'self'; connect-src 'self' https://api.openai.com https://api.anthropic.com https://generativelanguage.googleapis.com https://api.mistral.ai http://localhost:11434"

# Disable dangerous capabilities
dangerousDisableAssetCspModification = false
dangerousRemoteDomainIpcAccess = []
```

- The WebView cannot make arbitrary fetch calls. All AI API calls go through Tauri IPC → Rust → reqwest.
- `allowlist` is set to minimum required permissions only.
- No `eval()` or dynamic script loading.

### 14.3 Optional Database Encryption

Users can enable AES-256-GCM encryption of the SQLite database file via Settings > Security. When enabled:

1. User sets a passphrase.
2. A 256-bit key is derived using Argon2id (memory: 64MB, iterations: 3).
3. The database file is encrypted at rest using SQLCipher-compatible encryption via Rust.
4. The app prompts for the passphrase on launch before opening the database.
5. The passphrase is held in Rust memory only for the duration of the session — never written to disk.

### 14.4 Auto-Update Security

- Update checks contact GitHub Releases API only — no PromptVault-controlled server.
- Update manifests are signed with a Tauri update signing key (ed25519).
- The app verifies the signature before presenting the update to the user.
- The user downloads and installs the update manually — the app never executes downloaded code automatically.
- Users who distrust auto-update entirely can disable update checks in Settings.

### 14.5 No Telemetry Policy

The application contains zero telemetry, analytics, or error-reporting code by default. The open-source nature of the codebase makes this auditable and verifiable.

---

## 15. Plugin & Adapter System

### 15.1 Design Goals

- New AI providers and CLI tools can be added without modifying core application code.
- Community members can publish adapters as standalone packages.
- Adapters are written in Rust (for performance and safety) and loaded at compile time for the official build, or dynamically via a plugin manifest for community builds.

### 15.2 Official (Compiled-In) Adapters

For the official release, all first-party adapters are compiled into the binary. This is the simplest and most secure approach — no dynamic code loading, no supply chain attack surface.

```
src-tauri/src/adapters/
  ai/
    mod.rs          -- ProviderAdapter trait
    openai.rs
    anthropic.rs
    ollama.rs
    gemini.rs
    mistral.rs
    openai_compat.rs
  cli/
    mod.rs          -- CliAdapter trait
    claude_code.rs
    cursor.rs
    continue_dev.rs
    aider.rs
    copilot_cli.rs
```

### 15.3 Community Adapter Manifest (v2.0)

For the community plugin system (P2 feature), adapters are described in a manifest file and loaded via a Rust plugin interface:

```toml
# ~/.config/promptvault/plugins/my-adapter/manifest.toml
[adapter]
id = "my-custom-llm"
display_name = "My Custom LLM"
type = "ai_provider"          # or "cli_tool"
version = "1.0.0"
author = "community-dev"
requires_api_key = true
binary = "./my_adapter.so"    # compiled Rust dylib
```

---

## 16. Data Portability & Sync

### 16.1 Export Formats

**JSON (canonical)**
Full-fidelity export including all metadata, tags, collection structure, version history, and settings (excluding API keys). Suitable for full restore.

**Markdown bundle**
One `.md` file per prompt, organized in directories mirroring collection structure. Human-readable without PromptVault. Suitable for archiving or importing into Obsidian, Notion, etc.

**CSV**
Flat table of all prompts: id, title, body, tags, collection, created\_at, updated\_at. Suitable for spreadsheet analysis or migration tooling.

### 16.2 Opt-In Cloud Sync (User-Controlled)

PromptVault does not implement its own sync server. Instead, the sync feature works by writing the database backup file to a user-configured path. The user points that path at their Dropbox, iCloud Drive, or Google Drive folder. The cloud provider handles the syncing.

```
Settings > Sync > Backup path: ~/Dropbox/PromptVault/
Settings > Sync > Auto-backup interval: Every 1 hour
```

This means:
- PromptVault never touches any cloud API.
- The user controls which cloud provider (if any) handles their data.
- The user understands their data is in their Dropbox/iCloud and subject to those providers' terms — not PromptVault's.

### 16.3 GitHub Gist Export (Optional, P2)

Single prompts can be published as public or secret GitHub Gists using the user's own GitHub Personal Access Token, stored in the OS keychain. PromptVault makes a direct GitHub API call from Rust — no PromptVault server is involved. The Gist URL is copied to clipboard after creation.

---

## 17. Project Structure

```
promptvault/
├── src/                          # React frontend
│   ├── app/                      # Page-level components
│   │   ├── library/              # Prompt library view
│   │   ├── editor/               # Prompt editor view
│   │   ├── runner/               # Run prompt against AI
│   │   ├── injector/             # CLI injection flow
│   │   └── settings/             # Settings pages
│   ├── components/               # Reusable UI components
│   │   ├── ui/                   # Radix-based primitives
│   │   ├── prompt-card/
│   │   ├── collection-tree/
│   │   ├── variable-form/
│   │   └── diff-viewer/
│   ├── hooks/                    # Custom React hooks
│   ├── store/                    # Zustand stores
│   ├── lib/
│   │   ├── tauri.ts              # Typed wrappers for Tauri commands
│   │   └── utils.ts
│   └── main.tsx
│
├── src-tauri/                    # Rust core
│   ├── src/
│   │   ├── main.rs               # Entry point, Tauri app builder
│   │   ├── commands/             # Tauri IPC commands (called from React)
│   │   │   ├── prompts.rs
│   │   │   ├── collections.rs
│   │   │   ├── ai.rs
│   │   │   ├── cli.rs
│   │   │   └── settings.rs
│   │   ├── adapters/
│   │   │   ├── ai/               # AI provider adapters
│   │   │   └── cli/              # CLI tool adapters
│   │   ├── db/
│   │   │   ├── mod.rs            # DB connection + pool
│   │   │   ├── migrations.rs     # Migration runner
│   │   │   └── queries/          # Typed SQL queries
│   │   ├── keychain.rs           # OS keychain wrapper
│   │   └── crypto.rs             # Optional DB encryption
│   ├── migrations/               # SQL migration files
│   └── tauri.conf.json
│
├── cli/                          # Optional CLI companion (`promptvault` binary)
│   └── src/
│       └── main.rs               # clap-based CLI
│
├── tests/
│   ├── unit/                     # Vitest unit tests
│   └── e2e/                      # Playwright E2E tests
│
├── .github/
│   └── workflows/
│       ├── ci.yml                # Lint + test on every PR
│       └── release.yml           # Build + sign + publish on tag
│
├── CONTRIBUTING.md
├── LICENSE                       # MIT
└── README.md
```

---

## 18. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| OS keychain unavailable on headless Linux | API keys unreadable | Fall back to AES-encrypted file store; warn user prominently |
| SQLite corruption on unclean shutdown | Data loss | WAL mode + automatic backups every N minutes to a separate file |
| Ollama not running when user expects it | AI call fails silently | Health check polling; clear "Ollama offline" indicator in UI |
| Tauri WebView inconsistency across platforms | UI bugs on Linux | CI runs on all three platforms; automated Playwright E2E on each |
| Breaking CLI tool config file formats | Injection breaks after CLI update | Adapter version-pins and graceful fallback with clear error message |
| Community adapter ships malicious code | Supply chain attack | Compiled-in adapters only in official build; community adapters clearly marked as unofficial and require user opt-in |
| Large prompt library performance | SQLite slow queries | FTS5 index + WAL mode + benchmark tests in CI blocking merges that regress > 50ms search |
| User accidentally deletes DB file | Total data loss | Prominent backup reminders; scheduled auto-backup to user-chosen path |

---

## 19. Roadmap & Milestones

### M0 — Foundation (Weeks 1–3)
- Tauri v2 project scaffold with React + Vite + TypeScript
- SQLite setup with migrations, WAL mode, FTS5 index
- Core CRUD for prompts, collections, tags
- Basic UI: library view, editor, collection tree

### M1 — MVP Core (Weeks 4–7)
- Full-text search (FTS5, < 50ms)
- Version history with diff viewer
- Variable interpolation with fill form
- OS keychain integration for API keys
- OpenAI and Anthropic adapters (direct API calls from Rust)
- Ollama adapter (local models)
- Run prompt in-app with streaming response

### M2 — CLI Bridge (Weeks 8–10)
- Claude Code injection (CLAUDE.md)
- Cursor injection (.cursorrules)
- Continue.dev injection (config.json)
- Aider injection (.aider.conf.yml)
- Dry-run preview flow
- Injection log

### M3 — Polish & Portability (Weeks 11–13)
- Export: JSON, Markdown bundle, CSV
- Import: JSON restore
- Command palette (Cmd/Ctrl + K)
- Dark / light / system themes
- Keyboard shortcut coverage
- GitHub Actions: build + sign + release for macOS, Windows, Linux
- README, CONTRIBUTING, LICENSE

### M4 — Public Release v1.0 (Week 14)
- Public GitHub release
- Signed binaries for all three platforms
- Documentation site (static, no server — GitHub Pages)
- Community Discord / GitHub Discussions

### M5 — v1.1 (Weeks 15–22)
- Gemini + Mistral adapters
- Streaming response display
- Response history (local)
- Side-by-side model comparison
- Local REST API server (localhost only)
- CLI companion binary (`promptvault` command)
- Scheduled automatic local backups
- Bulk import/export

### M6 — v2.0 (Months 6–9)
- Plugin / adapter system for community providers
- Local semantic search (on-device embeddings via Ollama)
- Prompt chaining
- MCP server mode
- Optional GitHub Gist sharing
- Opt-in sync via user-controlled cloud folder

---

## Appendix A — Supported Platforms

| Platform | Min Version | Architecture | Status |
|----------|-------------|-------------|--------|
| macOS | 12 Monterey | Intel + Apple Silicon (universal binary) | P0 |
| Windows | 10 (build 19041) | x64 | P0 |
| Linux | Ubuntu 20.04 / Fedora 36 | x64 | P0 |
| Linux | Arch, NixOS | x64 | Best-effort / community |

---

## Appendix B — AI Provider Comparison

| Provider | Requires Key | Works Offline | Base URL Configurable | Notes |
|----------|-------------|--------------|----------------------|-------|
| OpenAI | Yes | No | No | Direct to api.openai.com |
| Anthropic | Yes | No | No | Direct to api.anthropic.com |
| Ollama | No | Yes | Yes (default localhost:11434) | Fully local; user installs separately |
| Gemini | Yes | No | No | Direct to Google AI |
| Mistral | Yes | No | No | Direct to api.mistral.ai |
| OpenAI-compat | Optional | Depends | Yes | LM Studio, Jan, vLLM, etc. |

---

## Appendix C — CLI Tool Compatibility

| Tool | Config Target | Strategy | Idempotent | Notes |
|------|--------------|---------|------------|-------|
| Claude Code | `CLAUDE.md` | Delimiter block | Yes | Global or per-project |
| Cursor | `.cursorrules` | Delimiter block | Yes | Per-project |
| Continue.dev | `~/.continue/config.json` | JSON key update | Yes | Parses JSON safely |
| Aider | `.aider.conf.yml` | YAML key update | Yes | Parses YAML safely |
| GitHub Copilot CLI | Shell alias | Clipboard snippet | N/A | No file write |

---

*This document supersedes PromptVault PRD/TRD v1.0 (cloud-first). All previous cloud architecture decisions are deprecated.*

*PromptVault — Your prompts. Your machine. Your keys.*
