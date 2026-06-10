mod data;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::Write;

#[derive(Parser)]
#[command(name = "tracker", about = "Project issue tracker", version)]
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
        /// Issue title
        title: String,
        /// Issue description
        #[arg(short, long)]
        description: Option<String>,
        /// Priority level
        #[arg(short, long, default_value = "medium")]
        priority: data::Priority,
        /// Label to apply; can be repeated
        #[arg(short, long)]
        label: Vec<String>,
    },
    /// List issues (excludes closed by default)
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<data::Status>,
        /// Filter by priority
        #[arg(short, long)]
        priority: Option<data::Priority>,
        /// Filter by label; can be repeated (OR logic)
        #[arg(short, long)]
        label: Vec<String>,
    },
    /// Show full details of an issue
    Show {
        /// Issue ID
        id: u32,
    },
    /// Delete an issue (prompts for confirmation)
    Delete {
        /// Issue ID
        id: u32,
    },
    /// Close an issue (sets status to closed; keeps the record)
    Close {
        /// Issue ID
        id: u32,
    },
    /// Edit an issue's description in $EDITOR
    Edit {
        /// Issue ID
        id: u32,
    },
    /// Update fields on an issue
    Update {
        /// Issue ID
        id: u32,
        /// New title
        #[arg(short, long)]
        title: Option<String>,
        /// New description (pass "" to clear)
        #[arg(short, long)]
        description: Option<String>,
        /// New status
        #[arg(short, long)]
        status: Option<data::Status>,
        /// New priority
        #[arg(short, long)]
        priority: Option<data::Priority>,
        /// Replace ALL labels (destructive). Use --add-label to append, or --clear-labels to remove all.
        #[arg(short, long, conflicts_with_all = ["add_label", "clear_labels"])]
        label: Vec<String>,
        /// Append one or more labels to the existing set; can be repeated
        #[arg(long, conflicts_with = "clear_labels")]
        add_label: Vec<String>,
        /// Remove all labels from the issue
        #[arg(long)]
        clear_labels: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Init => cmd_init(),
        Command::Create { title, description, priority, label } => cmd_create(title, description, priority, label),
        Command::List { status, priority, label } => cmd_list(status, priority, label),
        Command::Show { id } => cmd_show(id),
        Command::Delete { id } => cmd_delete(id),
        Command::Close { id } => cmd_close(id),
        Command::Edit { id } => cmd_edit(id),
        Command::Update { id, title, description, status, priority, label, add_label, clear_labels } => cmd_update(id, title, description, status, priority, label, add_label, clear_labels),
    };
    if let Err(e) = result {
        eprintln!("{} {e}", "error:".red().bold());
        std::process::exit(1);
    }
}

fn status_str(status: &data::Status) -> &'static str {
    match status {
        data::Status::Open => "open",
        data::Status::InProgress => "in-progress",
        data::Status::Done => "done",
        data::Status::Closed => "closed",
    }
}

fn status_colored(status: &data::Status, pad: usize) -> String {
    let s = format!("{:<width$}", status_str(status), width = pad);
    match status {
        data::Status::Open => s,
        data::Status::InProgress => s.cyan().to_string(),
        data::Status::Done => s.green().to_string(),
        data::Status::Closed => s.dimmed().to_string(),
    }
}

fn normalize_labels(labels: Vec<String>) -> Result<Vec<String>> {
    labels
        .into_iter()
        .map(|l| {
            let l = l.trim().to_lowercase();
            if l.is_empty() {
                anyhow::bail!("label cannot be empty");
            }
            Ok(l)
        })
        .collect()
}

fn cmd_init() -> Result<()> {
    let dir = data::tracker_dir()?;
    if dir.exists() {
        anyhow::bail!("tracker already initialized in this directory");
    }
    std::fs::create_dir_all(&dir)?;
    data::save_store(&data::Store { next_id: 1, issues: vec![] })?;
    println!("{}", format!("Initialized tracker in {}", dir.display()).green());
    Ok(())
}

fn cmd_create(title: String, description: Option<String>, priority: data::Priority, labels: Vec<String>) -> Result<()> {
    let title = title.trim().to_string();
    if title.is_empty() {
        anyhow::bail!("title cannot be empty");
    }
    let description = description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty());
    let labels = normalize_labels(labels)?;
    let mut store = data::load_store()?;
    let id = store.next_id;
    let issue = data::Issue {
        id,
        title: title.clone(),
        description,
        status: data::Status::Open,
        priority,
        labels,
        created_at: chrono::Utc::now(),
    };
    store.issues.push(issue);
    store.next_id += 1;
    data::save_store(&store)?;
    println!("{}", format!("Created issue #{id}: {title}").green());
    Ok(())
}

fn cmd_show(id: u32) -> Result<()> {
    let store = data::load_store()?;
    let issue = store.issues
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| anyhow::anyhow!("no issue with id #{id}"))?;

    let priority = match issue.priority {
        data::Priority::Critical => "critical".red().bold().underline().to_string(),
        data::Priority::High => "high".red().bold().to_string(),
        data::Priority::Medium => "medium".yellow().to_string(),
        data::Priority::Low => "low".dimmed().to_string(),
    };
    let labels = if issue.labels.is_empty() {
        "none".dimmed().to_string()
    } else {
        issue.labels.join(", ")
    };
    let created = issue.created_at.format("%Y-%m-%d %H:%M UTC").to_string();

    println!("{}", format!("#{} {}", issue.id, issue.title).bold());
    println!("Status:   {}", status_colored(&issue.status, 0));
    println!("Priority: {}", priority);
    println!("Labels:   {}", labels);
    if let Some(desc) = &issue.description {
        println!("Desc:     {}", desc);
    }
    println!("Created:  {}", created.dimmed());
    Ok(())
}

fn cmd_delete(id: u32) -> Result<()> {
    let mut store = data::load_store()?;
    let pos = store.issues
        .iter()
        .position(|i| i.id == id)
        .ok_or_else(|| anyhow::anyhow!("no issue with id #{id}"))?;

    println!(
        "{}",
        "Delete removes the issue permanently. To retire it while keeping the record, use `tracker close` instead.".dimmed()
    );
    print!("Delete issue #{id} \"{}\"? [y/N] ", store.issues[pos].title);
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes") {
        store.issues.remove(pos);
        data::save_store(&store)?;
        println!("{}", format!("Deleted issue #{id}").green());
    } else {
        println!("{}", "Aborted".dimmed());
    }
    Ok(())
}

fn cmd_close(id: u32) -> Result<()> {
    let mut store = data::load_store()?;
    let issue = store.issues
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| anyhow::anyhow!("no issue with id #{id}"))?;
    issue.status = data::Status::Closed;
    data::save_store(&store)?;
    println!("{}", format!("Closed issue #{id}").green());
    Ok(())
}

fn cmd_edit(id: u32) -> Result<()> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .map_err(|_| anyhow::anyhow!("no editor found — set $EDITOR or $VISUAL"))?;

    let mut store = data::load_store()?;
    let pos = store.issues
        .iter()
        .position(|i| i.id == id)
        .ok_or_else(|| anyhow::anyhow!("no issue with id #{id}"))?;

    let temp_path = std::env::temp_dir().join(format!("tracker-edit-{id}.md"));
    std::fs::write(&temp_path, store.issues[pos].description.as_deref().unwrap_or(""))
        .context("could not create temp file for editing")?;

    // `sh -c '<editor> "$1"' sh <path>` — passes the path as $1 so editors with
    // their own args (e.g. `code --wait`) work and the path can't be re-split.
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("{editor} \"$1\""))
        .arg("sh")
        .arg(&temp_path)
        .status()
        .context("could not launch editor")?;
    if !status.success() {
        let _ = std::fs::remove_file(&temp_path);
        anyhow::bail!("editor exited with an error; description unchanged");
    }

    let edited = std::fs::read_to_string(&temp_path).context("could not read edited description")?;
    let _ = std::fs::remove_file(&temp_path);

    let trimmed = edited.trim();
    store.issues[pos].description = if trimmed.is_empty() { None } else { Some(trimmed.to_string()) };
    data::save_store(&store)?;
    println!("{}", format!("Updated description for issue #{id}").green());
    Ok(())
}

fn cmd_update(id: u32, title: Option<String>, description: Option<String>, status: Option<data::Status>, priority: Option<data::Priority>, labels: Vec<String>, add_labels: Vec<String>, clear_labels: bool) -> Result<()> {
    if title.is_none() && description.is_none() && status.is_none() && priority.is_none()
        && labels.is_empty() && add_labels.is_empty() && !clear_labels {
        anyhow::bail!("at least one option required (--title, --description, --status, --priority, --label, --add-label, --clear-labels)");
    }
    let mut store = data::load_store()?;
    let issue = store.issues
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| anyhow::anyhow!("no issue with id #{id}"))?;
    if let Some(t) = title {
        let t = t.trim();
        if t.is_empty() {
            anyhow::bail!("title cannot be empty");
        }
        issue.title = t.to_string();
    }
    if let Some(d) = description {
        let d = d.trim();
        issue.description = if d.is_empty() { None } else { Some(d.to_string()) };
    }
    if let Some(s) = status {
        if issue.status == s {
            anyhow::bail!("issue #{id} is already {}", status_str(&s));
        }
        if !issue.status.can_transition_to(&s) {
            let allowed = issue.status.allowed_next().iter()
                .map(status_str)
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::bail!(
                "cannot move {} -> {}; valid transitions: {}",
                status_str(&issue.status), status_str(&s), allowed
            );
        }
        issue.status = s;
    }
    if let Some(p) = priority {
        issue.priority = p;
    }
    if clear_labels {
        issue.labels.clear();
    } else if !add_labels.is_empty() {
        for l in normalize_labels(add_labels)? {
            if !issue.labels.contains(&l) {
                issue.labels.push(l);
            }
        }
    } else if !labels.is_empty() {
        issue.labels = normalize_labels(labels)?;
    }
    data::save_store(&store)?;
    println!("{}", format!("Updated issue #{id}").green());
    Ok(())
}

fn cmd_list(status_filter: Option<data::Status>, priority_filter: Option<data::Priority>, label_filter: Vec<String>) -> Result<()> {
    let store = data::load_store()?;
    let has_other_filter = priority_filter.is_some() || !label_filter.is_empty();
    let mut visible: Vec<&data::Issue> = store.issues
        .iter()
        .filter(|i| match &status_filter {
            // Explicit --status shows exactly that status (closed included).
            Some(s) => &i.status == s,
            // A non-status filter widens to everything but closed.
            None if has_other_filter => i.status != data::Status::Closed,
            // No filters at all: only active work.
            None => matches!(i.status, data::Status::Open | data::Status::InProgress),
        })
        .filter(|i| priority_filter.as_ref().map_or(true, |p| &i.priority == p))
        .filter(|i| {
            label_filter.is_empty()
                || label_filter.iter().any(|l| i.labels.contains(&l.to_lowercase()))
        })
        .collect();

    if visible.is_empty() {
        let has_filters = status_filter.is_some() || priority_filter.is_some() || !label_filter.is_empty();
        if has_filters {
            println!("{}", "No issues match your filters.".dimmed());
        } else {
            println!("{}", "No open issues. Nice work!".green());
        }
        return Ok(());
    }

    visible.sort_by(|a, b| {
        // Highest priority first; within a bracket, done sinks to the bottom.
        b.priority.cmp(&a.priority).then_with(|| {
            let a_done = a.status == data::Status::Done;
            let b_done = b.status == data::Status::Done;
            a_done.cmp(&b_done)
        })
    });

    for issue in visible {
        let status = status_colored(&issue.status, 12);
        let priority = match issue.priority {
            data::Priority::Critical => format!("{:<8}", "critical").red().bold().underline().to_string(),
            data::Priority::High => format!("{:<8}", "high").red().bold().to_string(),
            data::Priority::Medium => format!("{:<8}", "medium").yellow().to_string(),
            data::Priority::Low => format!("{:<8}", "low").dimmed().to_string(),
        };
        let labels = if issue.labels.is_empty() {
            String::new()
        } else {
            format!(" [{}]", issue.labels.join(", ")).dimmed().to_string()
        };
        println!("#{:<4} {} {} {}{}", issue.id, status, priority, issue.title, labels);
    }
    Ok(())
}
