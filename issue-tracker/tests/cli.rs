use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// A `tracker` invocation scoped to `dir`, with color disabled for stable
/// string matching. Each call returns a fresh command (assert consumes it).
fn tracker(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("tracker").unwrap();
    cmd.current_dir(dir.path()).env("NO_COLOR", "1");
    cmd
}

/// A temp dir with an initialized tracker.
fn init_repo() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    tracker(&dir).arg("init").assert().success();
    dir
}

#[test]
fn create_lists_and_shows() {
    let dir = init_repo();
    tracker(&dir)
        .args(["create", "First issue", "--priority", "high", "--label", "bug"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#1"));
    tracker(&dir)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("First issue").and(predicate::str::contains("high")));
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bug"));
}

#[test]
fn ids_are_not_reused_after_delete() {
    let dir = init_repo();
    tracker(&dir).args(["create", "a"]).assert().success();
    tracker(&dir).args(["create", "b"]).assert().success(); // #2
    tracker(&dir)
        .args(["delete", "2"])
        .write_stdin("y\n")
        .assert()
        .success();
    // The freed #2 must not be reused.
    tracker(&dir)
        .args(["create", "c"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#3"));
}

#[test]
fn delete_with_yes_skips_prompt() {
    let dir = init_repo();
    tracker(&dir).args(["create", "a"]).assert().success();
    // No stdin provided; --yes must not block on the prompt.
    tracker(&dir)
        .args(["delete", "1", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted issue #1"));
    tracker(&dir).args(["show", "1"]).assert().failure();
}

#[test]
fn delete_aborts_on_no() {
    let dir = init_repo();
    tracker(&dir).args(["create", "a"]).assert().success();
    tracker(&dir)
        .args(["delete", "1"])
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Aborted"));
    tracker(&dir).args(["show", "1"]).assert().success();
}

#[test]
fn priority_short_alias_accepted() {
    let dir = init_repo();
    // `-p c` is accepted as an alias for `critical`.
    tracker(&dir).args(["create", "x", "-p", "c"]).assert().success();
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("critical"));
}

#[test]
fn close_already_closed_errors() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x"]).assert().success();
    tracker(&dir).args(["close", "1"]).assert().success();
    tracker(&dir)
        .args(["close", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already closed"));
}

#[test]
fn reopen_unclosees_only() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x"]).assert().success();
    tracker(&dir).args(["close", "1"]).assert().success();
    // closed -> open
    tracker(&dir)
        .args(["reopen", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Reopened issue #1"));
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status:").and(predicate::str::contains("open")));
    // Reopening a non-closed issue errors.
    tracker(&dir)
        .args(["reopen", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not closed"));
}

#[test]
fn invalid_status_transition_is_rejected() {
    let dir = init_repo();
    tracker(&dir).args(["create", "a"]).assert().success();
    tracker(&dir)
        .args(["update", "1", "--status", "done"]) // open -> done is a skip
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot move open -> done"));
    tracker(&dir).args(["update", "1", "--status", "in-progress"]).assert().success();
    tracker(&dir).args(["update", "1", "--status", "done"]).assert().success();
}

#[test]
fn default_list_shows_only_active() {
    let dir = init_repo();
    tracker(&dir).args(["create", "keep open"]).assert().success(); // 1
    tracker(&dir).args(["create", "will finish"]).assert().success(); // 2
    tracker(&dir).args(["update", "2", "--status", "in-progress"]).assert().success();
    tracker(&dir).args(["update", "2", "--status", "done"]).assert().success();
    tracker(&dir).args(["create", "will close"]).assert().success(); // 3
    tracker(&dir).args(["close", "3"]).assert().success();
    tracker(&dir).arg("list").assert().success().stdout(
        predicate::str::contains("keep open")
            .and(predicate::str::contains("will finish").not())
            .and(predicate::str::contains("will close").not()),
    );
}

#[test]
fn multi_value_status_filter_uses_or() {
    let dir = init_repo();
    tracker(&dir).args(["create", "stays open"]).assert().success(); // 1 open
    tracker(&dir).args(["create", "in prog"]).assert().success(); // 2
    tracker(&dir).args(["update", "2", "--status", "in-progress"]).assert().success();
    tracker(&dir).args(["create", "finished"]).assert().success(); // 3
    tracker(&dir).args(["update", "3", "--status", "in-progress"]).assert().success();
    tracker(&dir).args(["update", "3", "--status", "done"]).assert().success();
    // open OR done -> #1 and #3, but not the in-progress #2
    tracker(&dir)
        .args(["list", "--status", "open", "--status", "done"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("stays open")
                .and(predicate::str::contains("finished"))
                .and(predicate::str::contains("in prog").not()),
        );
}

#[test]
fn labels_are_deduped_on_write() {
    let dir = init_repo();
    // "bug"/"Bug"/"bug" all collapse to a single "bug".
    tracker(&dir)
        .args(["create", "x", "--label", "bug", "--label", "Bug", "--label", "bug"])
        .assert()
        .success();
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Labels:").and(predicate::str::contains("bug, bug").not()));
}

#[test]
fn list_filter_rejects_empty_label() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x", "--label", "bug"]).assert().success();
    // Empty filter errors like an empty write, instead of silently matching nothing.
    tracker(&dir)
        .args(["list", "--label", ""])
        .assert()
        .failure()
        .stderr(predicate::str::contains("label cannot be empty"));
}

#[test]
fn list_filter_trims_label() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x", "--label", "bug"]).assert().success();
    // Surrounding whitespace is trimmed on read, matching the stored "bug".
    tracker(&dir)
        .args(["list", "--label", "  bug  "])
        .assert()
        .success()
        .stdout(predicate::str::contains("x"));
}

#[test]
fn labels_are_normalized_and_filter_is_case_insensitive() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x", "--label", "BUG"]).assert().success();
    // Stored lowercased; a differently-cased filter still matches.
    tracker(&dir)
        .args(["list", "--label", "Bug"])
        .assert()
        .success()
        .stdout(predicate::str::contains("x").and(predicate::str::contains("BUG").not()));
}

#[test]
fn label_append_and_clear() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x", "--label", "bug"]).assert().success();
    // Append: duplicate "bug" is skipped, "Urgent" is normalized to "urgent".
    tracker(&dir)
        .args(["update", "1", "--add-label", "bug", "--add-label", "Urgent"])
        .assert()
        .success();
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bug, urgent"));
    // Clear removes them all.
    tracker(&dir).args(["update", "1", "--clear-labels"]).assert().success();
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Labels:").and(predicate::str::contains("none")));
    // The three label modes are mutually exclusive.
    tracker(&dir)
        .args(["update", "1", "--label", "a", "--clear-labels"])
        .assert()
        .failure();
}

#[test]
fn label_remove_single() {
    let dir = init_repo();
    tracker(&dir)
        .args(["create", "x", "--label", "bug", "--label", "urgent", "--label", "ui"])
        .assert()
        .success();
    // Remove one (case/space-insensitive); the others remain.
    tracker(&dir).args(["update", "1", "--remove-label", " Urgent "]).assert().success();
    tracker(&dir).args(["show", "1"]).assert().success().stdout(
        predicate::str::contains("bug, ui").and(predicate::str::contains("urgent").not()),
    );
    // Removing an absent label is a forgiving no-op (still succeeds).
    tracker(&dir).args(["update", "1", "--remove-label", "nope"]).assert().success();
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bug, ui"));
}

#[test]
fn description_set_and_clear() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x", "--description", "hello"]).assert().success();
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Description: hello"));
    tracker(&dir).args(["update", "1", "--description", ""]).assert().success();
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello").not());
}

#[test]
fn update_changes_title() {
    let dir = init_repo();
    tracker(&dir).args(["create", "old title"]).assert().success();
    tracker(&dir).args(["update", "1", "--title", "new title"]).assert().success();
    tracker(&dir).args(["show", "1"]).assert().success().stdout(
        predicate::str::contains("new title").and(predicate::str::contains("old title").not()),
    );
    // An empty/whitespace title is rejected.
    tracker(&dir)
        .args(["update", "1", "--title", "   "])
        .assert()
        .failure()
        .stderr(predicate::str::contains("title cannot be empty"));
}

#[test]
fn edit_updates_description_via_editor() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x"]).assert().success();
    // Fake editor: `cp <file>` overwrites the temp buffer with our content.
    let newdesc = dir.path().join("newdesc.txt");
    std::fs::write(&newdesc, "edited body").unwrap();
    tracker(&dir)
        .args(["edit", "1"])
        .env("EDITOR", format!("cp {}", newdesc.display()))
        .assert()
        .success();
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("edited body"));
}

#[test]
fn edit_cleans_up_temp_file() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x"]).assert().success();
    // Point the child's TMPDIR at an isolated dir so we can assert the edit
    // temp file (created there by tempfile) is gone afterward.
    let tmpdir = tempfile::tempdir().unwrap();
    let newdesc = dir.path().join("nd.txt");
    std::fs::write(&newdesc, "edited body").unwrap();
    tracker(&dir)
        .args(["edit", "1"])
        .env("EDITOR", format!("cp {}", newdesc.display()))
        .env("TMPDIR", tmpdir.path())
        .assert()
        .success();
    let leftover = std::fs::read_dir(tmpdir.path()).unwrap().count();
    assert_eq!(leftover, 0, "edit temp file was not cleaned up");
}

#[test]
fn edit_without_editor_errors() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x"]).assert().success();
    tracker(&dir)
        .args(["edit", "1"])
        .env_remove("EDITOR")
        .env_remove("VISUAL")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no editor found"));
}

#[test]
fn finds_tracker_from_subdirectory() {
    let dir = init_repo();
    tracker(&dir).args(["create", "root issue"]).assert().success();
    let sub = dir.path().join("a/b");
    std::fs::create_dir_all(&sub).unwrap();
    // Run from the nested subdir; it should walk up to the parent `.tracker`.
    Command::cargo_bin("tracker")
        .unwrap()
        .current_dir(&sub)
        .env("NO_COLOR", "1")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("root issue"));
}

#[test]
fn updated_at_recorded_only_after_modification() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x"]).assert().success();
    // Fresh issue: no updated_at, no Updated line.
    let json = std::fs::read_to_string(dir.path().join(".tracker/issues.json")).unwrap();
    assert!(!json.contains("updated_at"), "a freshly created issue should have no updated_at");
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated:").not());
    // After a modification it appears.
    tracker(&dir).args(["update", "1", "--priority", "high"]).assert().success();
    let json = std::fs::read_to_string(dir.path().join(".tracker/issues.json")).unwrap();
    assert!(json.contains("updated_at"));
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated:"));
}

#[test]
fn broken_pipe_exits_quietly() {
    let dir = init_repo();
    // Seed many issues directly (legacy array format is accepted) so the list
    // output overflows the pipe buffer and a closing reader reliably hits EPIPE.
    let mut json = String::from("[");
    for i in 1..=2000 {
        if i > 1 {
            json.push(',');
        }
        json.push_str(&format!(
            r#"{{"id":{i},"title":"issue number {i} with a bit of extra padding text","description":null,"status":"open","priority":"medium","labels":[],"created_at":"2026-01-01T00:00:00Z"}}"#
        ));
    }
    json.push(']');
    std::fs::write(dir.path().join(".tracker/issues.json"), json).unwrap();

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg("\"$BIN\" list | head -1")
        .env("BIN", env!("CARGO_BIN_EXE_tracker"))
        .env("NO_COLOR", "1")
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked") && !stderr.contains("Broken pipe"),
        "stderr should be clean on a broken pipe, got: {stderr}"
    );
}

#[test]
fn completions_carry_install_header() {
    let dir = tempfile::tempdir().unwrap();
    // The nushell output documents the easy-to-miss two-step / `use completions *`.
    Command::cargo_bin("tracker")
        .unwrap()
        .current_dir(dir.path())
        .args(["completions", "nushell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("use completions *").and(predicate::str::contains("save -f")));
}

#[test]
fn completions_generate_for_all_shells() {
    // Completions are static; they don't need an initialized tracker.
    let dir = tempfile::tempdir().unwrap();
    for sh in ["bash", "zsh", "fish", "nushell"] {
        Command::cargo_bin("tracker")
            .unwrap()
            .current_dir(dir.path())
            .args(["completions", sh])
            .assert()
            .success()
            .stdout(predicate::str::contains("tracker"));
    }
}

#[test]
fn concurrent_creates_do_not_lose_writes() {
    let dir = init_repo();
    let bin = env!("CARGO_BIN_EXE_tracker");
    let n = 20;
    // Fire off N creates at once; the write lock must serialize them.
    let children: Vec<_> = (0..n)
        .map(|i| {
            std::process::Command::new(bin)
                .args(["create", &format!("issue {i}")])
                .current_dir(dir.path())
                .env("NO_COLOR", "1")
                .stdout(std::process::Stdio::null())
                .spawn()
                .unwrap()
        })
        .collect();
    for mut c in children {
        assert!(c.wait().unwrap().success());
    }
    // Every create must survive the race — N issues, one line each (piped: no truncation).
    let out = Command::cargo_bin("tracker")
        .unwrap()
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .arg("list")
        .output()
        .unwrap();
    let count = String::from_utf8_lossy(&out.stdout).lines().count();
    assert_eq!(count, n, "expected {n} issues after concurrent creates, got {count}");
}

#[test]
fn list_renders_priority_order_and_sinks_done() {
    let dir = init_repo();
    // Shared label so a non-status filter makes done visible (per #7 widening).
    tracker(&dir).args(["create", "high-open", "-p", "high", "-l", "x"]).assert().success(); // 1
    tracker(&dir).args(["create", "crit-open", "-p", "critical", "-l", "x"]).assert().success(); // 2
    tracker(&dir).args(["create", "high-done", "-p", "high", "-l", "x"]).assert().success(); // 3
    tracker(&dir).args(["update", "3", "-s", "in-progress"]).assert().success();
    tracker(&dir).args(["update", "3", "-s", "done"]).assert().success();
    let out = Command::cargo_bin("tracker")
        .unwrap()
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .args(["list", "--label", "x"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let order: Vec<&str> = stdout
        .lines()
        .filter_map(|l| {
            if l.contains("crit-open") {
                Some("crit-open")
            } else if l.contains("high-open") {
                Some("high-open")
            } else if l.contains("high-done") {
                Some("high-done")
            } else {
                None
            }
        })
        .collect();
    // critical above high; within high, open before the sunk done.
    assert_eq!(order, vec!["crit-open", "high-open", "high-done"]);
}

#[test]
fn priority_filter_includes_done_excludes_closed() {
    let dir = init_repo();
    tracker(&dir).args(["create", "h-open", "-p", "high"]).assert().success(); // 1
    tracker(&dir).args(["create", "h-done", "-p", "high"]).assert().success(); // 2
    tracker(&dir).args(["update", "2", "-s", "in-progress"]).assert().success();
    tracker(&dir).args(["update", "2", "-s", "done"]).assert().success();
    tracker(&dir).args(["create", "h-closed", "-p", "high"]).assert().success(); // 3
    tracker(&dir).args(["close", "3"]).assert().success();
    tracker(&dir).args(["create", "m-open", "-p", "medium"]).assert().success(); // 4
    let out = Command::cargo_bin("tracker")
        .unwrap()
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .args(["list", "--priority", "high"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("h-open"), "open high should show");
    assert!(stdout.contains("h-done"), "done high should show (widening)");
    assert!(!stdout.contains("h-closed"), "closed must be excluded");
    assert!(!stdout.contains("m-open"), "other priorities filtered out");
}

#[test]
fn list_id_column_aligns_across_digit_widths() {
    let dir = init_repo();
    for i in 1..=10 {
        tracker(&dir).args(["create", &format!("t{i}")]).assert().success();
    }
    let out = Command::cargo_bin("tracker")
        .unwrap()
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .arg("list")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line1 = stdout.lines().find(|l| l.starts_with("#1 ")).unwrap();
    let line10 = stdout.lines().find(|l| l.starts_with("#10")).unwrap();
    // The id column widens to 2, so the priority column aligns on both rows.
    assert_eq!(
        line1.find("medium"),
        line10.find("medium"),
        "columns should align regardless of id digit width"
    );
}

#[test]
fn reopen_rejects_done() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x"]).assert().success();
    tracker(&dir).args(["update", "1", "-s", "in-progress"]).assert().success();
    tracker(&dir).args(["update", "1", "-s", "done"]).assert().success();
    // reopen only un-closes; a done issue is not reopenable (Option-A scope).
    tracker(&dir)
        .args(["reopen", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not closed"));
}

#[test]
fn label_modes_are_mutually_exclusive() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x", "-l", "a"]).assert().success();
    tracker(&dir)
        .args(["update", "1", "--remove-label", "a", "--add-label", "b"])
        .assert()
        .failure();
    tracker(&dir)
        .args(["update", "1", "--remove-label", "a", "--clear-labels"])
        .assert()
        .failure();
}

#[test]
fn commands_require_init() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("tracker")
        .unwrap()
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no tracker found"));
}
