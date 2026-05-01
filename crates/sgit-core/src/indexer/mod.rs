
pub mod embed;
pub mod git;

use indicatif::{ProgressBar, ProgressStyle};
use tracing::{debug, info, warn};

use crate::db::Store;
use crate::error::Result;
use embed::load_shared_model;
use git::read_commits;

pub struct IndexOptions {
    pub repo_path: std::path::PathBuf,
    pub incremental: bool,
}

impl Default for IndexOptions {
    fn default() -> Self {
        Self {
            repo_path: std::env::current_dir().unwrap_or_else(|_| ".".into()),
            incremental: true,
        }
    }
}

#[derive(Debug)]
pub struct IndexStats {
    pub total_commits: usize,
    pub new_commits: usize,
    pub skipped_commits: usize,
    pub db_path: std::path::PathBuf,
}

/// Run indexing pipeline.
pub async fn run(opts: IndexOptions) -> Result<IndexStats> {
    let store = Store::open(&opts.repo_path)?;
    info!(db = %store.db_path().display(), "Database opened");

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

    let model = load_shared_model()?;

    let pb = ProgressBar::new(commits_to_index.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} commits  ({eta} left)")
            .unwrap()
            .progress_chars("=>-"),
    );

    let messages: Vec<String> = commits_to_index
        .iter()
        .map(|c| c.message.clone())
        .collect();

    info!(count = messages.len(), "Starting batch embedding");
    let embeddings = model.embed_batch(&messages)?;
    pb.finish_and_clear();

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
