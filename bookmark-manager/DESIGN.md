# Bookmark Manager — Design Document

## What It Does

A personal bookmark manager that runs in a web browser. You can save links, write notes about them, organize them with tags, and search through them later. No login or accounts — it's just for you.

---

## Features

### Core

| Feature | Description |
|---|---|
| Save a bookmark | Enter a URL and title; the app stores it |
| Edit a bookmark | Change the title, notes, or tags on any saved bookmark |
| Delete a bookmark | Remove a bookmark permanently |
| Add notes | Write freeform text notes attached to any bookmark |
| Tag bookmarks | Add one or more short labels (e.g. `python`, `recipe`, `read-later`) |
| Search | Find bookmarks by typing into a search bar; matches title, URL, notes, and tags |
| Filter by tag | Click a tag to show only bookmarks with that tag |

### Out of Scope (to keep things simple)

- User accounts or login
- Sharing bookmarks with others
- Browser extension integration
- Importing/exporting (can add later)

---

## How It Looks

### Layout

```
┌─────────────────────────────────────────────────────┐
│  Bookmark Manager          [Search...]  [+ Add]     │  ← Header
├──────────────┬──────────────────────────────────────┤
│              │  ┌──────────────────────────────┐    │
│  Tags        │  │ Title                         │    │
│  ─────────   │  │ https://example.com           │    │
│  All (12)    │  │ Notes: ...          [python]  │    │
│  python (4)  │  └──────────────────────────────┘    │
│  recipe (3)  │                                       │
│  read-later  │  ┌──────────────────────────────┐    │
│  (2)         │  │ Title                         │    │
│              │  │ https://example.com           │    │
│              │  │ Notes: ...         [recipe]   │    │
│              │  └──────────────────────────────┘    │
└──────────────┴──────────────────────────────────────┘
```

- **Header**: app title, search bar, and an "Add Bookmark" button
- **Sidebar**: list of all tags with bookmark counts; click to filter
- **Main area**: bookmark cards, one per bookmark, showing title, URL, a snippet of notes, and tags

### Add / Edit Form (appears as a pop-up modal)

```
┌─────────────────────────────┐
│  Add Bookmark               │
│                             │
│  URL    [________________]  │
│  Title  [________________]  │
│  Notes  [________________]  │
│         [________________]  │
│  Tags   [________________]  │
│         (comma-separated)   │
│                             │
│         [Cancel]  [Save]    │
└─────────────────────────────┘
```

### Style

- Clean white background, neutral grays, one accent color (e.g. blue) for buttons and links
- Readable font size, comfortable spacing — nothing cramped
- Works on a laptop or desktop screen (no need to optimize for mobile yet)

---

## Technology

| Layer | Choice | Why |
|---|---|---|
| Structure | HTML | Standard, no setup needed |
| Styling | CSS | Plain CSS, no extra tools |
| Interactivity | JavaScript (vanilla) | No framework to learn first |
| Storage | `localStorage` | Built into every browser; no server needed |

No backend, no install, no build tools. Open `index.html` in a browser and it works. Data is saved automatically in your browser and persists between visits.

**One trade-off to know about:** `localStorage` is tied to the browser you use on the current computer. If you clear your browser data or switch browsers, your bookmarks will be gone. That's fine for a first project — you can always add an export feature later.

---

## Data Model

Bookmarks are stored in `localStorage` as a JSON array under the key `"bookmarks"`. Each bookmark is a plain JavaScript object:

```json
{
  "id": 1747123456789,
  "url": "https://example.com",
  "title": "Example Site",
  "notes": "Useful reference for X",
  "tags": ["python", "tutorial"],
  "createdAt": "2026-05-13T10:00:00Z"
}
```

The `id` is just a timestamp (milliseconds since 1970), which is unique enough for personal use. Tags are a plain array of strings.

---

## File Structure

```
bookmark-manager/
└── index.html    # The entire app: HTML structure, CSS styles, and JavaScript all in one file
```

That's it — one file.

---

## Build Order (Suggested)

1. **HTML skeleton** — header, sidebar, main area (static placeholder content, no logic yet)
2. **CSS** — style the layout and bookmark cards
3. **localStorage helpers** — write small JavaScript functions to load and save the bookmarks array
4. **Render bookmarks** — read from localStorage and display cards on the page
5. **Add form** — the modal for adding a new bookmark; save it to localStorage and refresh the list
6. **Edit and delete** — wire up edit/delete buttons on each card
7. **Search** — filter the displayed cards as the user types in the search bar
8. **Tag filtering** — populate the sidebar from saved tags; click to filter

---

## What You'll Learn

By building this project you'll touch:

- How to structure a web page with HTML
- How to style it with CSS
- How JavaScript can read and update a page without reloading
- How to store and retrieve data with `localStorage`
- How to work with JSON (the format used to save the bookmarks)
