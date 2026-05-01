
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name    = "sgit",
    version = env!("CARGO_PKG_VERSION"),
    about   = "Semantic git history search",
    long_about = None,
)]
pub struct Cli {
    #[arg(long, global = true, default_value = "warn")]
    pub log_level: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Prepare search index.
    Index {
        #[arg(long, default_value = "false")]
        full: bool,

        #[arg(long)]
        path: Option<std::path::PathBuf>,
    },

    /// Search git history.
    #[command(alias = "search")]
    Log {
        /// Search query.
        query: String,

        /// Max results.
        #[arg(short = 'n', long, default_value = "3")]
        count: usize,

        /// Filter by author.
        #[arg(long)]
        author: Option<String>,

        /// Filter by date (YYYY-MM-DD).
        #[arg(long)]
        after: Option<String>,

        /// Minimum relevance score.
        #[arg(long, default_value = "0.15")]
        min_score: f32,

        /// Show relevance scores.
        #[arg(long, default_value = "false")]
        show_scores: bool,
    },

    /// Show index status.
    Status,

    /// Set up git post-commit hook.
    Hook {
        #[arg(long)]
        path: Option<std::path::PathBuf>,
    },

    /// Internal installation command.
    #[command(hide = true)]
    Install,

    /// Download latest version.
    Update,

    /// Remove sgit and data.
    Uninstall,
}