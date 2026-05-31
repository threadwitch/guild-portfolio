mod data;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::Write;

#[derive(Parser)]
#[command(name = "tracker", about = "Project issue tracker")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a tracker in the current directory
    Init,
    /// Create a new issue
    Create {
        title: String,
        #[arg(long, default_value = "medium")]
        priority: data::Priority,
        #[arg(long)]
        label: Vec<String>,
    },
    /// List issues (excludes closed by default)
    List {
        #[arg(long)]
        status: Option<data::Status>,
        #[arg(long)]
        priority: Option<data::Priority>,
        #[arg(long)]
        label: Vec<String>,
    },
    /// Show full details of an issue
    Show { id: u32 },
    /// Delete an issue
    Delete { id: u32 },
    /// Update an issue
    Update {
        id: u32,
        #[arg(long)]
        status: Option<data::Status>,
        #[arg(long)]
        priority: Option<data::Priority>,
        #[arg(long)]
        label: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Init => cmd_init(),
        Command::Create { title, priority, label } => cmd_create(title, priority, label),
        Command::List { status, priority, label } => cmd_list(status, priority, label),
        Command::Show { id } => cmd_show(id),
        Command::Delete { id } => cmd_delete(id),
        Command::Update { id, status, priority, label } => cmd_update(id, status, priority, label),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn cmd_init() -> Result<()> {
    let dir = data::tracker_dir();
    if dir.exists() {
        anyhow::bail!("tracker already initialized in this directory");
    }
    std::fs::create_dir_all(&dir)?;
    data::save_issues(&[])?;
    println!("Initialized tracker in {}", dir.display());
    Ok(())
}

fn status_str(status: &data::Status) -> &'static str {
    match status {
        data::Status::Open => "open",
        data::Status::InProgress => "in-progress",
        data::Status::Done => "done",
        data::Status::Closed => "closed",
    }
}

fn normalize_labels(labels: Vec<String>) -> Result<Vec<String>> {
    labels
        .into_iter()
        .map(|l| {
            let l = l.trim().to_string();
            if l.is_empty() {
                anyhow::bail!("label cannot be empty");
            }
            Ok(l)
        })
        .collect()
}

fn cmd_create(title: String, priority: data::Priority, labels: Vec<String>) -> Result<()> {
    let title = title.trim().to_string();
    if title.is_empty() {
        anyhow::bail!("title cannot be empty");
    }
    let labels = normalize_labels(labels)?;
    let mut issues = data::load_issues()?;
    let id = issues.iter().map(|i| i.id).max().unwrap_or(0) + 1;
    let issue = data::Issue {
        id,
        title: title.clone(),
        description: None,
        status: data::Status::Open,
        priority,
        labels,
        created_at: chrono::Utc::now(),
    };
    issues.push(issue);
    data::save_issues(&issues)?;
    println!("Created issue #{id}: {title}");
    Ok(())
}

fn cmd_show(id: u32) -> Result<()> {
    let issues = data::load_issues()?;
    let issue = issues
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| anyhow::anyhow!("no issue with id #{id}"))?;

    let priority = match issue.priority {
        data::Priority::High => "high".red().bold().to_string(),
        data::Priority::Medium => "medium".yellow().to_string(),
        data::Priority::Low => "low".dimmed().to_string(),
    };
    let labels = if issue.labels.is_empty() {
        "none".to_string()
    } else {
        issue.labels.join(", ")
    };
    let created = issue.created_at.format("%Y-%m-%d %H:%M UTC").to_string();

    println!("#{} {}", issue.id, issue.title);
    println!("Status:   {}", status_str(&issue.status));
    println!("Priority: {}", priority);
    println!("Labels:   {}", labels);
    if let Some(desc) = &issue.description {
        println!("Desc:     {}", desc);
    }
    println!("Created:  {}", created);
    Ok(())
}

fn cmd_delete(id: u32) -> Result<()> {
    let mut issues = data::load_issues()?;
    let pos = issues
        .iter()
        .position(|i| i.id == id)
        .ok_or_else(|| anyhow::anyhow!("no issue with id #{id}"))?;

    print!("Delete issue #{id} \"{}\"? [y/N] ", issues[pos].title);
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes") {
        issues.remove(pos);
        data::save_issues(&issues)?;
        println!("Deleted issue #{id}");
    } else {
        println!("Aborted");
    }
    Ok(())
}

fn cmd_update(id: u32, status: Option<data::Status>, priority: Option<data::Priority>, labels: Vec<String>) -> Result<()> {
    if status.is_none() && priority.is_none() && labels.is_empty() {
        anyhow::bail!("at least one option required (--status, --priority, --label)");
    }
    let labels = if labels.is_empty() { None } else { Some(normalize_labels(labels)?) };
    let mut issues = data::load_issues()?;
    let issue = issues
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| anyhow::anyhow!("no issue with id #{id}"))?;
    if let Some(s) = status {
        issue.status = s;
    }
    if let Some(p) = priority {
        issue.priority = p;
    }
    if let Some(l) = labels {
        issue.labels = l;
    }
    data::save_issues(&issues)?;
    println!("Updated issue #{id}");
    Ok(())
}

fn cmd_list(status_filter: Option<data::Status>, priority_filter: Option<data::Priority>, label_filter: Vec<String>) -> Result<()> {
    let issues = data::load_issues()?;
    let mut visible: Vec<&data::Issue> = issues
        .iter()
        .filter(|i| match &status_filter {
            Some(s) => &i.status == s,
            None => i.status != data::Status::Closed,
        })
        .filter(|i| priority_filter.as_ref().map_or(true, |p| &i.priority == p))
        .filter(|i| {
            label_filter.is_empty()
                || label_filter.iter().any(|l| i.labels.contains(l))
        })
        .collect();

    if visible.is_empty() {
        println!("No open issues. Nice work!");
        return Ok(());
    }

    visible.sort_by(|a, b| b.priority.cmp(&a.priority));

    for issue in visible {
        let status = status_str(&issue.status);
        let priority = match issue.priority {
            data::Priority::High => format!("{:<8}", "high").red().bold().to_string(),
            data::Priority::Medium => format!("{:<8}", "medium").yellow().to_string(),
            data::Priority::Low => format!("{:<8}", "low").dimmed().to_string(),
        };
        let labels = if issue.labels.is_empty() {
            String::new()
        } else {
            format!(" [{}]", issue.labels.join(", "))
        };
        println!("#{:<4} {:<12} {} {}{}", issue.id, status, priority, issue.title, labels);
    }
    Ok(())
}
