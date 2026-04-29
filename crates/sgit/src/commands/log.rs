/// Handler for `sgit log` — the main user-facing command.
///
/// Flow:
/// 1. Load the embedding model (fast after first load, uses LRU cache)
/// 2. Embed the user's query
/// 3. Score every commit by cosine similarity
/// 4. Print results in a git-log-like format
use anyhow::Result;
use colored::Colorize;
use sgit_core::{
    indexer::embed::load_shared_model,
    search::{search, SearchOptions},
    SgitError,
};
use tracing::debug;

pub async fn run(
    query: String,
    count: usize,
    author: Option<String>,
    after: Option<String>,
    min_score: f32,
    show_scores: bool,
) -> Result<()> {
    debug!(query = %query, count, "Running log command");

    // Load model — uses LRU cache for repeated calls in the same session
    let model = match load_shared_model() {
        Ok(m) => m,
        Err(SgitError::ModelLoad(e)) => {
            eprintln!("\n  {} Could not load embedding model: {}\n", "Error:".red().bold(), e);
            std::process::exit(1);
        }
        Err(e) => return Err(e.into()),
    };

    let opts = SearchOptions {
        top_n: count,
        min_score,
        author_filter: author.clone(),
        after_date: after.clone(),
    };

    let results = match search(&query, &model, &opts) {
        Ok(r) => r,
        Err(SgitError::IndexNotFound) => {
            eprintln!(
                "\n  {} Run {} first to build the search index.\n",
                "No index found.".yellow().bold(),
                "sgit index".cyan()
            );
            std::process::exit(1);
        }
        Err(SgitError::NoResults) => {
            println!(
                "\n  {} No commits matched \"{}\".\n  Try broader terms.\n",
                "No results.".yellow(),
                query
            );
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    if results.is_empty() {
        println!(
            "\n  {} No commits matched \"{}\".\n",
            "→".dimmed(),
            query.yellow()
        );
        return Ok(());
    }

    // Print header
    println!(
        "\n{} {}\n",
        "Results for:".dimmed(),
        query.yellow().bold()
    );

    // Print each result in a git-log-like format
    for result in &results {
        if show_scores {
            print!(
                "  {} ",
                format!("[{:.0}%]", result.score * 100.0).dimmed()
            );
        }
        println!(
            "{} {} {} {}",
            result.sha.yellow(),
            result.date.dimmed(),
            result.author.cyan(),
            result.message.bold()
        );
    }

    println!();

    // Show filter hints if filters were active
    if author.is_some() || after.is_some() {
        let mut active = vec![];
        if let Some(ref a) = author {
            active.push(format!("author={}", a));
        }
        if let Some(ref d) = after {
            active.push(format!("after={}", d));
        }
        println!("  {} {}\n", "Filters:".dimmed(), active.join(", ").dimmed());
    }

    Ok(())
}