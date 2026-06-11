# Issue Tracker - Design Document

## Purpose
A personal issue tracker CLI for managing project tasks. Single user, runs in
the terminal, scoped to a project directory. The tracker is located by walking
up from the current directory to the nearest `.tracker/` (like `git`), so
commands work from any subdirectory of a project.

## Data Model

An issue has the following fields:
- `id`: sequential integer, assigned on creation from a persisted counter — never reused, even after deletion
- `title`: required, non-empty string
- `description`: optional string
- `status`: `open` | `in-progress` | `done` | `closed`
- `priority`: `low` | `medium` | `high` | `critical`
- `labels`: array of strings, normalized to lowercase and de-duplicated
- `created_at`: ISO 8601 timestamp (UTC)
- `updated_at`: optional ISO 8601 timestamp; absent until the issue is first modified after creation

### Storage
Data is stored in `.tracker/issues.json` as a JSON object:

```json
{ "next_id": 12, "issues": [ /* ... */ ] }
```

- `next_id` is the counter for the next id; it only increases, so ids are never reused.
- Writes are **atomic** (written to a temp file, then `rename`d over `issues.json`) and **serialized** across concurrent invocations by an advisory `flock` on `.tracker/lock`, so racing writes cannot lose data.
- A legacy bare-array `issues.json` (no `next_id`) is still read; `next_id` is synthesized from the highest id and the file is upgraded to the object form on the next write.
- `.tracker/lock` and `.tracker/issues.json.tmp` are runtime artifacts (git-ignored); `issues.json` is the tracked data.

## Status Flow
Statuses move through a validated, **adjacent-only** flow (no skipping):

- `open` ↔ `in-progress` ↔ `done`
- any status → `closed`
- `closed` → `open` (reopen)

`update --status` enforces these transitions and rejects no-ops. `close` and
`reopen` are first-class shortcuts: `close` sets `closed` from any state;
`reopen` returns a **closed** issue to `open` (closed only — a `done` issue is
not reopened directly). `done` and `closed` are distinct: `done` = completed,
`closed` = retired / won't-pursue, preserved for the record rather than deleted.

## Commands

- `tracker init` — initialize `.tracker/` in the current directory
- `tracker create "Fix the login bug" [-d TEXT] [-p high] [-l bug]...` — create an issue
- `tracker list [filters]` — list issues (see visibility rules below)
- `tracker show <id>` — full details of an issue
- `tracker update <id> [options]` — change fields (see below)
- `tracker close <id>` / `tracker reopen <id>` — retire / un-retire an issue
- `tracker delete <id> [-y|--yes|--force]` — remove permanently (prompts unless `--yes`); prefer `close` for record-keeping
- `tracker edit <id>` — edit the description in `$EDITOR` (falls back to `$VISUAL`; errors if neither is set)
- `tracker completions <bash|zsh|fish|nushell>` — print a shell completion script (with install instructions in a comment header)

### `list` filters and visibility
- `-s/--status` and `-p/--priority` are repeatable (OR within each type); different filter types combine as AND. `-l/--label` is repeatable (OR), with labels normalized to match storage.
- Visibility:
  - **No filters** → only `open` and `in-progress`.
  - **Explicit `--status`** → exactly those statuses (may include `closed`).
  - **A non-status filter** (`--priority`/`--label`) → everything *except* `closed` (so `done` is included).
- Sort: by priority (`critical` → `low`); within a bracket, `done` sinks to the bottom.

### `update` options
- `-t/--title` — set the title
- `-d/--description` — set the description (pass `""` to clear)
- `-s/--status` — change status (validated transition)
- `-p/--priority` — change priority
- `-l/--label` — replace all labels
- `--add-label` — append a label (repeatable)
- `--remove-label` — remove a specific label (repeatable)
- `--clear-labels` — remove all labels

The four label options are mutually exclusive. At least one option is required.

### Input conventions
- Short flag aliases: `-t -d -s -p -l -y`.
- Priority value aliases: `c` / `h` / `m` / `l` (critical / high / medium / low).
- Labels are trimmed, lowercased, and de-duplicated on both write and filter; empty labels are rejected.

## Output
- Colored, column-aligned list output. Column widths derive from the data (the widest status/priority value name, the widest id) rather than fixed constants.
- Long titles truncate to the terminal width — measured in display columns (unicode-aware) — when stdout is a TTY; piped output is left full so scripts get complete data.
- A broken stdout pipe (e.g. `tracker list | head`) exits cleanly instead of panicking.
- Empty-state messages (e.g. "No open issues. Nice work!").

## Technology
- **Rust**, `clap` (derive) for the CLI, `serde` / `serde_json` for storage, `chrono` for timestamps, `colored` for output, `anyhow` for errors.
- `clap_complete` (+ `clap_complete_nushell`) for completions; `terminal_size` for width; `unicode-width` for display-column truncation; `tempfile` for safe temp files; `libc` `flock` for the write lock.
- Tested with Rust's built-in harness: unit tests for the data layer (ordering, transitions, storage parsing/migration, formatting) and integration tests that drive the built binary (`assert_cmd`), including concurrency, pipe handling, and completion generation.

## Out of Scope
- Multiple users or sharing
- Due dates or calendar integration
- Sub-issues or hierarchy (flat for now)
- Time tracking

> Note: build history and review notes live in `PROCESS.md`; the issue backlog
> itself is tracked in `.tracker/`. This document describes the current design.
