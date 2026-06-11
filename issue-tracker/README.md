# tracker

A small, single-user issue tracker for the terminal. It keeps a project's
issues in a `.tracker/issues.json` file scoped to the project directory — and,
like `git`, finds the tracker by walking up from your current directory.

## Features
- Create, list, show, update, and delete issues
- A validated status flow: `open → in-progress → done`, plus `closed`
- Priorities `low | medium | high | critical`, with colored, sorted output
- Labels you can add / remove / replace / clear, normalized and de-duplicated
- First-class `close` / `reopen`, and `edit` to write descriptions in `$EDITOR`
- Flexible filtering (status / priority / label) and shell completions

## Install
From the project directory:
```sh
cargo install --path .
```
This builds and installs the `tracker` binary into `~/.cargo/bin`.

## Quick start
```sh
tracker init                                    # set up .tracker/ here
tracker create "Fix the login bug" -p high -l bug
tracker create "Write the README" -p medium -l docs
tracker list
```
```
#1 open        high     Fix the login bug [bug]
#2 open        medium   Write the README [docs]
```
Move an issue through its lifecycle:
```sh
tracker update 1 -s in-progress
tracker update 1 -s done
tracker show 1
```

## Commands
| Command | Description |
| --- | --- |
| `tracker init` | Initialize `.tracker/` in the current directory |
| `tracker create "<title>" [opts]` | Create an issue |
| `tracker list [filters]` | List issues (see below) |
| `tracker show <id>` | Show full details of an issue |
| `tracker update <id> [opts]` | Change fields on an issue |
| `tracker close <id>` / `tracker reopen <id>` | Retire / un-retire an issue |
| `tracker delete <id> [-y]` | Delete permanently (prompts unless `-y`) |
| `tracker edit <id>` | Edit the description in `$EDITOR` |
| `tracker completions <shell>` | Print a shell completion script |

Run `tracker <command> --help` for full options.

### Creating and updating
```sh
tracker create "title" -d "a description" -p critical -l bug -l urgent
tracker update 3 -t "new title" -d ""           # rename; clear description
tracker update 3 --add-label ui --remove-label urgent
```
- Short flags: `-t -d -s -p -l -y`. Priority also accepts aliases `c h m l`.
- Label modes on `update` are mutually exclusive: `--label` (replace all),
  `--add-label`, `--remove-label`, `--clear-labels`.

### Listing and filtering
```sh
tracker list                       # active work (open + in-progress)
tracker list -s done               # exactly the done issues
tracker list -p high -p critical   # high OR critical (any status but closed)
tracker list -l bug                # issues labeled bug
```
By default `list` shows only `open` and `in-progress`, sorted by priority
(`critical` first; `done` sinks to the bottom of its bracket). An explicit
`--status` shows exactly those statuses (including `closed`); any other filter
widens to everything except `closed`.

### Status flow
```
open  ⇄  in-progress  ⇄  done
any status  ──→  closed  ──→  open      (close / reopen)
```
`update --status` only allows adjacent moves. `close` works from any status;
`reopen` brings a closed issue back to `open`. `done` means finished; `closed`
means retired / won't-pursue — kept for the record, so prefer it over `delete`.

## Shell completions
`tracker completions <bash|zsh|fish|nushell>` prints a completion script, each
with install instructions in a header comment. For example, bash:
```sh
source <(tracker completions bash)
```

## Data & storage
Issues live in `.tracker/issues.json` (a `{ next_id, issues }` object). Writes
are atomic and lock-serialized, so concurrent invocations won't clobber each
other. The data file is meant to be committed; `.tracker/lock` and `*.tmp` are
runtime artifacts (git-ignore them).

## Development
```sh
cargo build
cargo test
```
See `DESIGN.md` for the design and `PROCESS.md` for build history.
