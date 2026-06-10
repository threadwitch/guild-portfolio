# Issue Tracker - Design Document

## Purpose
A personal issue tracker CLI for managing project tasks.
Single user, runs in the terminal, scoped to a project directory.

## Data Model

An issue has the following fields:
- `id`: sequential integer, assigned on creation
- `title`: string
- `description`: optional string
- `status`: `open` | `in-progress` | `done` | `closed`
- `priority`: `low` | `medium` | `high`
- `labels`: array of strings (e.g. `bug`, `feature`)
- `created_at`: ISO 8601 timestamp

Data is stored in `.tracker/issues.json` in the project directory.

## Commands

- `tracker init` — initialize `.tracker/` in the current directory
- `tracker create "Fix the login bug" [--priority high] [--label bug]`
- `tracker list` — show all non-closed issues, sorted by priority
- `tracker list --status done` — filter by status
- `tracker list --label bug --label feature` — filter by label (OR logic)
- `tracker list --status closed` — show closed issues
- `tracker show <id>` — show full details of an issue
- `tracker update <id> [--status in-progress] [--priority high] [--label bug]`
- `tracker delete <id>` — remove an issue

Notes:
- `--label` on `update` replaces the label list entirely
- Filters can be combined; multiple `--label` values use OR logic
- `closed` is excluded from all default list output

## Technology
- Rust
- CLI interface using subcommands

## Out of Scope
- Multiple users or sharing
- Due dates or calendar integration
- Subissues or hierarchy (keep it flat for now)
- Time tracking
- `--label-append` / `--label-remove` (deferred)

## Build order
1. **Core:** Set up the project. Create issues with a title and list them. Save to a JSON file. We only need `tracker create "title"` and `tracker list` as commands so far.
2. **Status flow:** Add status (open → in progress → done, closed). `tracker update <id> --status done` to change it. `tracker list` only shows open by default.
3. **Priority:** Add priority levels. Sort the list by priority. Show priority in the output with color or markers.
4. **Labels:** Add labels to issues. Display them in the list. Filter with `tracker list --label` bug.
5. **Compound filtering:** Make status, priority, and label filters work together. `tracker list --status open --priority high --label bug` shows high-priority open bugs. Waiting until the data structure is fleshed out should lead to less refactoring for the filter code.
6. **Detail and delete:** `tracker show <id>` shows full details including description and timestamps. `tracker delete <id>` with confirmation.
7. **Polish:** Helpful error messages, colored output other than priority, a `--help` flag that explains every command, empty-state messages ("No open issues. Nice work!").