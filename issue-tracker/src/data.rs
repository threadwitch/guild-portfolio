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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
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
