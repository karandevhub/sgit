
use anyhow::Result;
use colored::Colorize;
use sgit_core::{run_index, IndexOptions};

pub async fn run(full: bool, path: Option<std::path::PathBuf>) -> Result<()> {
    let repo_path = path.unwrap_or_else(|| {
        std::env::current_dir().expect("Cannot read current directory")
    });

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
        incremental: !full,
    };

    match run_index(opts).await {
        Ok(stats) => {
            if stats.new_commits == 0 && stats.total_commits > 0 {
                println!(
                    "\n  {} Index is already up to date ({} commits).\n",
                    "✓".green().bold(),
                    stats.total_commits.to_string().bold()
                );
            } else {
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
            eprintln!("\n  {} {}\n", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}