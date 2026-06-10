use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Open,
    #[value(name = "in-progress")]
    InProgress,
    Done,
    Closed,
}

impl Status {
    /// Statuses this one may transition to (adjacent-only flow).
    pub fn allowed_next(&self) -> &'static [Status] {
        match self {
            Status::Open => &[Status::InProgress, Status::Closed],
            Status::InProgress => &[Status::Open, Status::Done, Status::Closed],
            Status::Done => &[Status::InProgress, Status::Closed],
            Status::Closed => &[Status::Open],
        }
    }

    pub fn can_transition_to(&self, next: &Status) -> bool {
        self.allowed_next().contains(next)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    /// Explicit sort rank: higher is more urgent. This — not the enum's
    /// declaration order — defines how priorities sort, so reordering or
    /// inserting variants can't silently change the sort.
    fn rank(&self) -> u8 {
        match self {
            Priority::Low => 0,
            Priority::Medium => 1,
            Priority::High => 2,
            Priority::Critical => 3,
        }
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
    pub priority: Priority,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl Issue {
    /// Record that the issue was just modified.
    pub fn touch(&mut self) {
        self.updated_at = Some(Utc::now());
    }
}

pub fn tracker_dir() -> anyhow::Result<PathBuf> {
    Ok(std::env::current_dir()
        .context("could not determine current directory")?
        .join(".tracker"))
}

/// Locate an existing `.tracker` by walking up from the current directory,
/// so commands work from any subdirectory of a project (like git).
fn find_tracker_dir() -> anyhow::Result<PathBuf> {
    let start = std::env::current_dir().context("could not determine current directory")?;
    for dir in start.ancestors() {
        let candidate = dir.join(".tracker");
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    anyhow::bail!("no tracker found in this directory or any parent — run `tracker init` first");
}

pub fn issues_path() -> anyhow::Result<PathBuf> {
    Ok(find_tracker_dir()?.join("issues.json"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    pub next_id: u32,
    pub issues: Vec<Issue>,
}

pub fn load_store() -> anyhow::Result<Store> {
    let path = issues_path()?;
    if !path.exists() {
        anyhow::bail!("tracker is initialized but its issues file is missing");
    }
    let contents = fs::read_to_string(&path).context("could not read issues file")?;
    parse_store(&contents)
}

/// Parse the issues file contents: accepts the current `{ next_id, issues }`
/// object, or a legacy bare array (synthesizing `next_id` from the max id).
fn parse_store(contents: &str) -> anyhow::Result<Store> {
    if let Ok(store) = serde_json::from_str::<Store>(contents) {
        return Ok(store);
    }
    let issues: Vec<Issue> = serde_json::from_str(contents)
        .context("issues file is corrupt — the JSON is invalid or has an unexpected structure")?;
    let next_id = issues.iter().map(|i| i.id).max().unwrap_or(0) + 1;
    Ok(Store { next_id, issues })
}

pub fn save_store(store: &Store) -> anyhow::Result<()> {
    let contents = serde_json::to_string_pretty(store)?;
    let path = issues_path()?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, contents).context("could not write issues file")?;
    fs::rename(&tmp, &path).context("could not replace issues file")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_orders_low_to_critical() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    #[test]
    fn status_transitions_follow_adjacent_flow() {
        assert!(Status::Open.can_transition_to(&Status::InProgress));
        assert!(Status::InProgress.can_transition_to(&Status::Done));
        assert!(Status::Done.can_transition_to(&Status::Closed));
        assert!(Status::Closed.can_transition_to(&Status::Open));
        // rejected: forward skip, backward jump, and no-op
        assert!(!Status::Open.can_transition_to(&Status::Done));
        assert!(!Status::Done.can_transition_to(&Status::Open));
        assert!(!Status::Open.can_transition_to(&Status::Open));
    }

    #[test]
    fn parses_new_object_format() {
        let store = parse_store(r#"{ "next_id": 5, "issues": [] }"#).unwrap();
        assert_eq!(store.next_id, 5);
        assert!(store.issues.is_empty());
    }

    #[test]
    fn migrates_legacy_array_and_synthesizes_next_id() {
        let json = r#"[
            {"id":1,"title":"a","description":null,"status":"open","priority":"low","labels":[],"created_at":"2026-01-01T00:00:00Z"},
            {"id":7,"title":"b","description":null,"status":"done","priority":"high","labels":["bug"],"created_at":"2026-01-02T00:00:00Z"}
        ]"#;
        let store = parse_store(json).unwrap();
        assert_eq!(store.issues.len(), 2);
        assert_eq!(store.next_id, 8); // max id (7) + 1
    }

    #[test]
    fn empty_legacy_array_starts_next_id_at_one() {
        let store = parse_store("[]").unwrap();
        assert_eq!(store.next_id, 1);
    }

    #[test]
    fn corrupt_json_errors() {
        assert!(parse_store("not json").is_err());
    }

    #[test]
    fn legacy_issue_without_updated_at_loads_as_none() {
        let json = r#"[{"id":1,"title":"a","description":null,"status":"open","priority":"low","labels":[],"created_at":"2026-01-01T00:00:00Z"}]"#;
        let store = parse_store(json).unwrap();
        assert!(store.issues[0].updated_at.is_none());
    }
}
