mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cli.log_level)))
        .init();

    // Route command.
    match cli.command {
        Commands::Index { full, path } => {
            commands::index::run(full, path).await?;
        }
        Commands::Log {
            query,
            count,
            author,
            after,
            min_score,
            show_scores,
        } => {
            commands::log::run(query, count, author, after, min_score, show_scores).await?;
        }
        Commands::Status => {
            commands::status::run().await?;
        }
        Commands::Hook { path } => {
            commands::hook::run(path).await?;
        }
        Commands::Install => {
            commands::install::run().await?;
        }
        Commands::Update => {
            commands::update::run().await?;
        }
        Commands::Uninstall => {
            commands::uninstall::run().await?;
        }
    }

    Ok(())
}
