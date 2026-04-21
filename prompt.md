# PromptVault — Minimalist UI Design Prompt

> Use this prompt with an AI design tool (Claude, v0, Lovable, Bolt, etc.) to generate the frontend UI for PromptVault — a local-first, privacy-respecting desktop prompt manager built with Tauri + React + Tailwind CSS.

---

## App Overview

Design a **minimalist, dark-mode-first** desktop application UI called **PromptVault**. It is a local AI prompt manager — think Obsidian meets a chat client. The app runs as a native desktop window (no browser chrome). Typography should be clean, spacing generous, and every element purposeful. No decorative gradients, no heavy shadows. Use a neutral gray palette with one accent color (blue or indigo).

---

## Layout Structure

Three-column resizable layout with a bottom panel:

```
┌─────────────┬──────────────────┬───────────────────────────────────┐
│  Sidebar    │   Prompt List    │         Main Editor Panel         │
│  (nav)      │   (scroll list)  │  Title bar + Body + Action bar    │
│             │                  ├───────────────────────────────────┤
│             │                  │       AI Runner / Chat Panel      │
│             │                  │       (resizable, bottom)         │
└─────────────┴──────────────────┴───────────────────────────────────┘
```

All three column widths are user-resizable via drag handles. The AI panel height is also resizable.

---

## Column 1 — Sidebar

**Width:** ~256px, collapsible

**Contents (top to bottom):**
- App logo / wordmark — "PromptVault" in small, semibold text with a subtle icon (vault or lock)
- Navigation items:
  - All Prompts (default selected)
  - Pinned
  - Recent
  - Archived
- Collections section header with a "+" add button inline
  - Each collection: colored dot + name + prompt count badge
  - Collections are nestable (show indented sub-collections)
- Tags section (collapsed by default) — pill-shaped colored tags
- Bottom actions (pinned to footer):
  - Import / Export buttons (icon + label)
  - Settings gear icon
  - Theme toggle (sun/moon)

**Style:** No borders between items. Active item uses a subtle filled background. Muted text for counts. Thin separator between nav, collections, and tags sections.

---

## Column 2 — Prompt List

**Width:** ~288px, scrollable

**Contents:**
- Each prompt card (no card borders — use hover background only):
  - Pin icon (if pinned) — top-left, small
  - Title — semibold, truncated to one line
  - Body preview — 2-line truncated, muted smaller text
  - Last-updated timestamp — tiny, bottom-right, muted
- Selected state: subtle accent-colored left border + light accent background
- Empty state: centered muted text "No prompts yet"
- Loading state: skeleton placeholders

---

## Column 3 — Main Editor Panel

### Action Bar (top strip)
Compact icon-button row:
- Pin toggle (filled when active)
- Duplicate
- Copy to clipboard
- Delete (destructive red on hover)

### Title Row
- Editable title (click to edit inline) — large, semibold
- "Edit" / "Save" / "Cancel" buttons appear when in edit mode

### Prompt Body
- Monospace font
- Click-to-edit (no explicit edit button needed — clicking the body enters edit mode)
- In view mode: raw text display, no border
- In edit mode: minimal textarea, subtle border, monospace

### AI Runner Panel (resizable bottom section)

**Three tabs:**
1. **Run** — Single-shot prompt execution
2. **Chat** — Multi-turn conversation
3. **A/B Test** — Side-by-side model comparison

**Tab style:** Underline indicator tabs, not pill tabs. No background fill on inactive tabs.

#### Run Tab Contents:
- Provider dropdown + Model dropdown/input (inline, compact, `h-8`)
- Run button (primary)
- Attach file button (paperclip icon, ghost)
- "Configure keys →" link (far right, muted)
- Attached file chips: rounded-full, file type icon, name, size, × remove
- Image warning banner (amber) when vision files attached
- Response area: bordered box, monospace-ish rendered markdown, copy button top-right
- Streaming indicator: animated pulse dot next to provider/model label

#### Chat Tab Contents:
- Provider + model selectors (same compact row as Run tab)
- System prompt context bar at top: "System: [truncated prompt text]"
- Action buttons in system bar: **Import** · **Export JSON** · **Export MD** · **Clear** (shown only when messages exist)
- Message list (scrollable):
  - User messages: right-aligned, accent background bubble
  - Assistant messages: left-aligned, muted background, small bot avatar icon
  - Markdown rendered in assistant messages
- Loading indicator: animated bot avatar with spinner
- Error message: inline destructive-colored box
- Input area: auto-expanding textarea + Send button (icon only, `h-9`)
  - Enter to send, Shift+Enter for newline
  - Placeholder: "Type a message… (Enter to send, Shift+Enter for newline)"

#### A/B Test Tab Contents:
- Two side-by-side panels (split equally)
- Each panel has its own provider + model selector
- "Run Both" button centered above
- Output areas below each selector, same style as Run tab response area

---

## Global UI Components

### Command Palette (`Ctrl+K`)
- Centered modal, max-w-lg, top-aligned (pt-24)
- Backdrop: `bg-black/50`
- Input at top with search icon
- Two sections: "Prompts" (fuzzy search results) and "Commands"
- Each command row: name + keyboard shortcut (right-aligned, muted)
- Keyboard navigable (arrow keys + Enter)

### Error / Toast Banner
- Full-width strip below header, `bg-destructive/10 text-destructive`
- Dismiss × button on right
- Also used for success messages (same slot, green-ish text for 4s then auto-dismiss)

### Settings Modal
- Standard dialog overlay
- Sections: API Keys, Theme, Model Defaults, About
- Each API key field: label + masked password input + show/hide toggle

---

## Design Tokens

| Token | Value |
|-------|-------|
| Background | `#0f0f0f` (dark) / `#ffffff` (light) |
| Surface | `#1a1a1a` / `#f9f9f9` |
| Border | `#2a2a2a` / `#e5e5e5` |
| Muted text | `#6b7280` |
| Foreground | `#f0f0f0` / `#111111` |
| Accent | `#6366f1` (indigo-500) |
| Destructive | `#ef4444` |
| Font (UI) | Inter, system-ui |
| Font (code/body) | `JetBrains Mono`, `Fira Code`, monospace |
| Border radius | `8px` panels, `6px` buttons, `4px` inputs |
| Base font size | `14px` |

---

## Interaction Details

- **Resize handles:** 4px-wide invisible strips between columns. On hover: accent-colored. On drag: cursor changes to `col-resize` / `row-resize`.
- **Hover states:** Background shift only — no borders appearing on hover (except resize handles).
- **Focus states:** 2px offset ring in accent color on all interactive elements.
- **Transitions:** `150ms ease` on all color/bg transitions. No transform animations except loading spinner.
- **Scrollbars:** Thin, auto-hidden, matching surface color.
- **Empty states:** Centered, muted, no illustration — just text.

---

## Key Screens to Design

1. **Default state** — Empty prompt list, "No prompts yet" center state in editor
2. **Prompt selected, view mode** — Body text visible, AI panel in Run tab
3. **Prompt selected, edit mode** — Title + body in edit state
4. **Chat tab active** — Full conversation with mixed user/assistant messages
5. **Chat export/import buttons visible** — When messages > 0
6. **Command palette open** — Overlay with search + results
7. **Settings modal open**
8. **Light mode variant** — Same layout, light tokens

---

## What NOT to Include

- No splash screen or onboarding wizard
- No heavy card shadows or glassmorphism
- No emoji in the UI (except user-assigned collection icons)
- No animations beyond spinner and 150ms transitions
- No bottom navigation bar or mobile layout
- No floating action buttons
- No tooltips with arrows — use `title` attribute only
