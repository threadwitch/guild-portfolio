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
fn description_set_and_clear() {
    let dir = init_repo();
    tracker(&dir).args(["create", "x", "--description", "hello"]).assert().success();
    tracker(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
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
