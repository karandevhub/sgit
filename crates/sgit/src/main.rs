mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// This is the entry point of the sgit application.
// We use #[tokio::main] because sgit performs asynchronous tasks, 
// like downloading models or indexing many commits in parallel.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Parse the command-line arguments (e.g., 'sgit log "fix bugs"')
    let cli = Cli::parse();

    // 2. Set up logging. 
    // This allows us to see what the app is doing behind the scenes.
    // By default, it's quiet, but you can see more by setting RUST_LOG=debug.
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cli.log_level)))
        .init();

    // 3. Route the command to the right function.
    // Based on what you typed (index, log, status, etc.), we call 
    // the corresponding code in our 'commands' module.
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
