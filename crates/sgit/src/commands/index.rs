// This module handles the 'sgit index' command.
// It acts as the "glue" between the command-line and the core logic.

use anyhow::Result;
use colored::Colorize;
use sgit_core::{run_index, IndexOptions};

pub async fn run(full: bool, path: Option<std::path::PathBuf>) -> Result<()> {
    // 1. Figure out which folder we are indexing. 
    // Defaults to the one you are currently in.
    let repo_path = path.unwrap_or_else(|| {
        std::env::current_dir().expect("Cannot read current directory")
    });

    // 2. Print a nice header to show the user what's happening.
    println!(
        "\n{} {}",
        "sgit index".bold().cyan(),
        repo_path.display().to_string().dimmed()
    );

    if full {
        println!(
            "  {} Rebuilding full index from scratch...",
            "→".yellow()
        );
    }

    let opts = IndexOptions {
        repo_path,
        incremental: !full, // If not 'full', we only index new commits.
    };

    // 3. Call the core logic to actually do the indexing.
    // All the heavy lifting (reading Git, running AI) happens inside 'run_index'.
    match run_index(opts).await {
        Ok(stats) => {
            if stats.new_commits == 0 && stats.total_commits > 0 {
                println!(
                    "\n  {} Index is already up to date ({} commits).\n",
                    "✓".green().bold(),
                    stats.total_commits.to_string().bold()
                );
            } else {
                // Print a summary of what was indexed.
                println!(
                    "\n  {} Indexed {} new commits  ({} total, {} skipped)",
                    "✓".green().bold(),
                    stats.new_commits.to_string().green().bold(),
                    stats.total_commits.to_string().bold(),
                    stats.skipped_commits.to_string().dimmed(),
                );
                println!(
                    "  {} {}\n",
                    "DB:".dimmed(),
                    sgit_core::config::display_path(&stats.db_path).dimmed()
                );
                println!(
                    "  Run {} to search your history.\n",
                    "sgit log \"your query\"".cyan()
                );
            }
            Ok(())
        }
        Err(sgit_core::SgitError::NoRepository(path)) => {
            eprintln!(
                "\n  {} {}\n",
                "Error:".red().bold(),
                format!("Not a git repository: {}", path)
            );
            std::process::exit(1);
        }
        Err(e) => {
            // If anything goes wrong, show a clear error message.
            eprintln!("\n  {} {}\n", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}