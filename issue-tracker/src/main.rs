mod data;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

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
    },
    /// List issues (excludes closed)
    List,
    /// Update an issue
    Update {
        id: u32,
        #[arg(long)]
        status: Option<data::Status>,
        #[arg(long)]
        priority: Option<data::Priority>,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Init => cmd_init(),
        Command::Create { title, priority } => cmd_create(title, priority),
        Command::List => cmd_list(),
        Command::Update { id, status, priority } => cmd_update(id, status, priority),
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

fn cmd_create(title: String, priority: data::Priority) -> Result<()> {
    let title = title.trim().to_string();
    if title.is_empty() {
        anyhow::bail!("title cannot be empty");
    }
    let mut issues = data::load_issues()?;
    let id = issues.iter().map(|i| i.id).max().unwrap_or(0) + 1;
    let issue = data::Issue {
        id,
        title: title.clone(),
        description: None,
        status: data::Status::Open,
        priority,
        labels: vec![],
        created_at: chrono::Utc::now(),
    };
    issues.push(issue);
    data::save_issues(&issues)?;
    println!("Created issue #{id}: {title}");
    Ok(())
}

fn cmd_update(id: u32, status: Option<data::Status>, priority: Option<data::Priority>) -> Result<()> {
    if status.is_none() && priority.is_none() {
        anyhow::bail!("at least one option required (--status, --priority)");
    }
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
    data::save_issues(&issues)?;
    println!("Updated issue #{id}");
    Ok(())
}

fn cmd_list() -> Result<()> {
    let issues = data::load_issues()?;
    let mut visible: Vec<&data::Issue> = issues
        .iter()
        .filter(|i| i.status != data::Status::Closed)
        .collect();

    if visible.is_empty() {
        println!("No open issues. Nice work!");
        return Ok(());
    }

    visible.sort_by(|a, b| b.priority.cmp(&a.priority));

    for issue in visible {
        let status = match issue.status {
            data::Status::Open => "open",
            data::Status::InProgress => "in-progress",
            data::Status::Done => "done",
            data::Status::Closed => "closed",
        };
        let priority = match issue.priority {
            data::Priority::High => format!("{:<8}", "high").red().bold().to_string(),
            data::Priority::Medium => format!("{:<8}", "medium").yellow().to_string(),
            data::Priority::Low => format!("{:<8}", "low").dimmed().to_string(),
        };
        println!("#{:<4} {:<12} {} {}", issue.id, status, priority, issue.title);
    }
    Ok(())
}
