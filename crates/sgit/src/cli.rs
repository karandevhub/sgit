/// CLI definition using clap's derive macro.
///
/// Each command is a variant of the `Commands` enum.
/// Clap auto-generates --help, --version, and argument parsing from this.
use clap::{Parser, Subcommand};

/// sgit — semantic search for your git history.
///
/// Find commits with natural language instead of grep patterns.
///
/// Examples:
///   sgit index                          # build the search index
///   sgit log "authentication bug"       # find auth-related commits
///   sgit log "database migration" -n 5  # top 5 migration commits
///   sgit log "cache fix" --author alice # filtered by author
#[derive(Parser, Debug)]
#[command(
    name    = "sgit",
    version = env!("CARGO_PKG_VERSION"),
    about   = "Semantic git history search",
    long_about = None,
)]
pub struct Cli {
    /// Set log level: error, warn, info, debug, trace
    /// Can also be set with RUST_LOG env var (e.g. RUST_LOG=debug)
    #[arg(long, global = true, default_value = "warn")]
    pub log_level: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build (or update) the semantic search index for this repository.
    ///
    /// Run this once in any git repo. Re-run after major commit activity.
    /// The post-commit git hook runs this automatically if installed.
    ///
    /// Examples:
    ///   sgit index              # incremental — only new commits
    ///   sgit index --full       # full rebuild from scratch
    Index {
        /// Rebuild the entire index from scratch (ignores existing data)
        #[arg(long, default_value = "false")]
        full: bool,

        /// Path to the git repository (defaults to current directory)
        #[arg(long)]
        path: Option<std::path::PathBuf>,
    },

    /// Search git history with a natural language query.
    ///
    /// Finds commits semantically similar to your query — no exact match needed.
    ///
    /// Examples:
    ///   sgit log "when did auth break"
    ///   sgit log "stripe payment refactor" -n 5
    ///   sgit log "database schema change" --author john
    ///   sgit log "performance fix" --after 2024-01-01
    #[command(alias = "search")]
    Log {
        /// Natural language search query
        query: String,

        /// Number of results to show (default: 10)
        #[arg(short = 'n', long, default_value = "10")]
        count: usize,

        /// Filter results by author name (partial match)
        #[arg(long)]
        author: Option<String>,

        /// Only show commits after this date (format: YYYY-MM-DD)
        #[arg(long)]
        after: Option<String>,

        /// Minimum relevance score 0.0–1.0 (default: 0.15)
        #[arg(long, default_value = "0.15")]
        min_score: f32,

        /// Show raw relevance scores
        #[arg(long, default_value = "false")]
        show_scores: bool,
    },

    /// Show index statistics for the current repository.
    Status,

    /// Install the git post-commit hook to keep the index auto-updated.
    ///
    /// After this, `sgit index --incremental` runs automatically on every commit.
    Hook {
        /// Path to the git repository (defaults to current directory)
        #[arg(long)]
        path: Option<std::path::PathBuf>,
    },

    /// Install sgit binary to ~/.local/bin (or /usr/local/bin on some systems).
    /// Called automatically by install.sh after downloading the binary.
    #[command(hide = true)]
    Install,

    /// Check for and apply updates to sgit.
    Update,

    /// Completely uninstall sgit, removing the binary, data, and cache.
    Uninstall,
}