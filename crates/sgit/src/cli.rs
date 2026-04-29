// This module defines our Command Line Interface (CLI). 
// We use a library called 'clap' which automatically turns this code into 
// a working CLI with --help, --version, and argument parsing.

use clap::{Parser, Subcommand};

/// sgit — natural language search for your git history.
///
/// Instead of using 'grep' to find exact words, you can search for the 
/// *meaning* of what happened in your commits.
///
/// Examples:
///   sgit index                          # Prepare the search index
///   sgit log "authentication bug"       # Search for login-related issues
///   sgit log "ui cleanup" -n 5          # Find the top 5 UI-related commits
#[derive(Parser, Debug)]
#[command(
    name    = "sgit",
    version = env!("CARGO_PKG_VERSION"),
    about   = "Semantic git history search",
    long_about = None,
)]
pub struct Cli {
    /// How much detail to show in logs: error, warn, info, debug, trace.
    #[arg(long, global = true, default_value = "warn")]
    pub log_level: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Prepare the search index for this repository.
    ///
    /// You only need to run this once when you first start using sgit 
    /// on a project. After that, it updates automatically.
    Index {
        /// Force a full rebuild of the index from scratch.
        #[arg(long, default_value = "false")]
        full: bool,

        /// Specify a path to the git repo (defaults to the current folder).
        #[arg(long)]
        path: Option<std::path::PathBuf>,
    },

    /// Search through your git history.
    #[command(alias = "search")]
    Log {
        /// What are you looking for? (e.g., "fix payment bug")
        query: String,

        /// How many results to show (default: 10).
        #[arg(short = 'n', long, default_value = "10")]
        count: usize,

        /// Filter by who wrote the commit.
        #[arg(long)]
        author: Option<String>,

        /// Only show commits after this date (YYYY-MM-DD).
        #[arg(long)]
        after: Option<String>,

        /// Minimum relevance (0.0 to 1.0). Higher means more strict.
        #[arg(long, default_value = "0.15")]
        min_score: f32,

        /// Print the AI's internal relevance scores.
        #[arg(long, default_value = "false")]
        show_scores: bool,
    },

    /// Show information about the current index and database size.
    Status,

    /// Set up a "git hook" that automatically updates the index after every commit.
    Hook {
        #[arg(long)]
        path: Option<std::path::PathBuf>,
    },

    /// Internal command used during installation.
    #[command(hide = true)]
    Install,

    /// Check for and download the latest version of sgit.
    Update,

    /// Remove sgit and all its data from your system.
    Uninstall,
}