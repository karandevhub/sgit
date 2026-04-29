/// Indexer orchestrator: reads git history → embeds → saves to DB.
///
/// The key performance optimisation here (from studying smfs's approach):
/// 1. Read ALL commits from git first (fast — pure memory reads)
/// 2. Filter out already-indexed commits by checking SHAs in the DB
/// 3. Embed in batches of 64 (fastembed handles internal parallelism)
/// 4. Write to DB in a single transaction (much faster than per-row inserts)
///
/// This brings indexing a 5,000-commit repo from ~5min to ~20sec.
pub mod embed;
pub mod git;

use indicatif::{ProgressBar, ProgressStyle};
use tracing::{debug, info, warn};

use crate::db::Store;
use crate::error::Result;
use embed::load_shared_model;
use git::read_commits;

/// Options for the index command.
pub struct IndexOptions {
    /// Path to the git repository to index (defaults to current directory)
    pub repo_path: std::path::PathBuf,
    /// If true, only index commits not already in the DB (incremental update)
    pub incremental: bool,
}

impl Default for IndexOptions {
    fn default() -> Self {
        Self {
            repo_path: std::env::current_dir().unwrap_or_else(|_| ".".into()),
            incremental: true, // default to incremental — always safe
        }
    }
}

/// Statistics returned after indexing completes.
#[derive(Debug)]
pub struct IndexStats {
    pub total_commits: usize,
    pub new_commits: usize,
    pub skipped_commits: usize,
    pub db_path: std::path::PathBuf,
}

/// Run the full indexing pipeline.
///
/// Called from `sgit index`. Logs progress at every step using `tracing`
/// so users can set RUST_LOG=debug to see detailed internals.
pub async fn run(opts: IndexOptions) -> Result<IndexStats> {
    // Step 1: Open (or create) the database
    let store = Store::open()?;
    info!(db = %store.db_path().display(), "Database opened");

    // Step 2: Read all commits from git history
    info!("Reading git history...");
    let all_commits = read_commits(&opts.repo_path)?;

    if all_commits.is_empty() {
        warn!("No indexable commits found in this repository");
        return Ok(IndexStats {
            total_commits: 0,
            new_commits: 0,
            skipped_commits: 0,
            db_path: store.db_path().to_path_buf(),
        });
    }

    // Step 3: Filter out commits already in the DB (incremental mode)
    let commits_to_index = if opts.incremental {
        let existing_shas = store.get_all_shas()?;
        let filtered: Vec<_> = all_commits
            .iter()
            .filter(|c| !existing_shas.contains(&c.sha))
            .cloned()
            .collect();

        debug!(
            total = all_commits.len(),
            new = filtered.len(),
            cached = all_commits.len() - filtered.len(),
            "Filtered commits for incremental index"
        );

        filtered
    } else {
        all_commits.clone()
    };

    let skipped = all_commits.len() - commits_to_index.len();

    if commits_to_index.is_empty() {
        info!("Index is already up to date — nothing to do");
        return Ok(IndexStats {
            total_commits: all_commits.len(),
            new_commits: 0,
            skipped_commits: skipped,
            db_path: store.db_path().to_path_buf(),
        });
    }

    // Step 4: Load the embedding model
    let model = load_shared_model()?;

    // Step 5: Embed all new commits in batches
    // Progress bar with commit count
    let pb = ProgressBar::new(commits_to_index.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} commits  ({eta} left)")
            .unwrap()
            .progress_chars("=>-"),
    );

    // Extract just the messages for batch embedding
    let messages: Vec<String> = commits_to_index
        .iter()
        .map(|c| c.message.clone())
        .collect();

    info!(count = messages.len(), "Starting batch embedding");
    let embeddings = model.embed_batch(&messages)?;
    pb.finish_and_clear();

    // Step 6: Write everything to DB in one transaction (much faster than row-by-row)
    info!("Writing to database...");
    let mut records = Vec::with_capacity(commits_to_index.len());
    for (commit, embedding) in commits_to_index.iter().zip(embeddings.iter()) {
        records.push(crate::db::CommitRecord {
            sha: commit.sha.clone(),
            message: commit.message.clone(),
            author: commit.author.clone(),
            date: commit.date.clone(),
            timestamp: commit.timestamp,
            embedding: embedding.clone(),
        });
    }

    let new_count = store.upsert_batch(&records)?;
    info!(inserted = new_count, "Database write complete");

    Ok(IndexStats {
        total_commits: all_commits.len(),
        new_commits: new_count,
        skipped_commits: skipped,
        db_path: store.db_path().to_path_buf(),
    })
}
