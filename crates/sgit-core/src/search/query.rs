
use rayon::prelude::*;
use tracing::debug;

use crate::db::{CommitRecord, Store};
use crate::error::{Result, SgitError};
use crate::indexer::embed::EmbedModel;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub score: f32,
}

pub struct SearchOptions {
    pub top_n: usize,
    pub min_score: f32,
    pub author_filter: Option<String>,
    pub after_date: Option<String>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            top_n: 3,
            min_score: 0.15,
            author_filter: None,
            after_date: None,
        }
    }
}

/// Perform semantic search.
pub fn search(
    query: &str,
    model: &EmbedModel,
    opts: &SearchOptions,
    repo_path: &std::path::Path,
) -> Result<Vec<SearchResult>> {
    // Check for existing index.
    let store = Store::open(repo_path)?;
    let count = store.count()?;

    if count == 0 {
        return Err(SgitError::IndexNotFound);
    }

    debug!(query = %query, "Starting semantic search");

    // Embed search query.
    let query_vec = model.embed_query(query)?;

    // Load all indexed commits.
    let commits = store.load_all()?;
    debug!(loaded = commits.len(), "Loaded commits for scoring");

    // Compare vectors in parallel.
    let mut scored: Vec<(f32, &CommitRecord)> = commits
        .par_iter()
        .map(|commit| {
            // Compute cosine similarity.
            let score = cosine_similarity(&query_vec, &commit.embedding);
            (score, commit)
        })
        // Filter by relevance.
        .filter(|(score, _)| *score >= opts.min_score)
        .collect();

    // Apply filters.
    if let Some(ref author) = opts.author_filter {
        let author_lower = author.to_lowercase();
        scored.retain(|(_, c)| c.author.to_lowercase().contains(&author_lower));
    }

    if let Some(ref after) = opts.after_date {
        scored.retain(|(_, c)| c.date.as_str() >= after.as_str());
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let results = scored
        .into_iter()
        .take(opts.top_n)
        .map(|(score, commit)| SearchResult {
            sha: commit.sha.clone(),
            message: commit.message.clone(),
            author: commit.author.clone(),
            date: commit.date.clone(),
            score,
        })
        .collect();

    Ok(results)
}

/// Compute cosine similarity between two vectors.
#[inline]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have the same length");

    let mut dot = 0.0f32;
    let mut mag_a = 0.0f32;
    let mut mag_b = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot   += x * y;
        mag_a += x * x;
        mag_b += y * y;
    }

    let mag_a = mag_a.sqrt();
    let mag_b = mag_b.sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_identical_vectors() {
        let v = vec![1.0, 2.0, 3.0];
        let score = cosine_similarity(&v, &v);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_orthogonal_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let score = cosine_similarity(&a, &b);
        assert!(score.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_zero_vector() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 2.0, 3.0];
        let score = cosine_similarity(&a, &b);
        assert_eq!(score, 0.0);
    }
}