use missions_tui::{registry, tui};

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "missions-tui", about = "Read-only mission control panel")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Manage the repository registry
    Repo {
        #[command(subcommand)]
        action: RepoAction,
    },
}

#[derive(Debug, Subcommand)]
enum RepoAction {
    /// Add a repository to the registry
    Add {
        /// Path to the repository
        path: PathBuf,
    },
    /// List all registered repositories
    List,
    /// Remove a repository from the registry
    Remove {
        /// Path to the repository to remove
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Repo { action }) => {
            let config = registry::config_dir()?;
            match action {
                RepoAction::Add { path } => {
                    registry::add(&config, &path)?;
                    println!("Added: {}", path.display());
                }
                RepoAction::List => {
                    let reg = registry::load(&config);
                    if reg.repos.is_empty() {
                        println!("No repositories registered.");
                    } else {
                        for entry in &reg.repos {
                            let mission_info = if let Some((id, dir)) =
                                registry::discover_active_mission(&entry.path)
                            {
                                let state = registry::read_mission_state(&dir)
                                    .unwrap_or_else(|| "unknown".to_string());
                                format!("  [mission: {id} / {state}]")
                            } else {
                                String::new()
                            };
                            println!("{}{}", entry.path.display(), mission_info);
                        }
                    }
                }
                RepoAction::Remove { path } => {
                    registry::remove(&config, &path)?;
                    println!("Removed: {}", path.display());
                }
            }
        }
        None => {
            tui::run_app()?;
        }
    }

    Ok(())
}
