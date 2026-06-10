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
}

pub fn tracker_dir() -> anyhow::Result<PathBuf> {
    Ok(std::env::current_dir()
        .context("could not determine current directory")?
        .join(".tracker"))
}

pub fn issues_path() -> anyhow::Result<PathBuf> {
    Ok(tracker_dir()?.join("issues.json"))
}

pub fn load_issues() -> anyhow::Result<Vec<Issue>> {
    let path = issues_path()?;
    if !path.exists() {
        anyhow::bail!("no tracker found in current directory — run `tracker init` first");
    }
    let contents = fs::read_to_string(&path)
        .context("could not read issues file")?;
    serde_json::from_str(&contents)
        .context("issues file is corrupt — the JSON is invalid or has an unexpected structure")
}

pub fn save_issues(issues: &[Issue]) -> anyhow::Result<()> {
    let contents = serde_json::to_string_pretty(issues)?;
    let path = issues_path()?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, contents).context("could not write issues file")?;
    fs::rename(&tmp, &path).context("could not replace issues file")?;
    Ok(())
}
