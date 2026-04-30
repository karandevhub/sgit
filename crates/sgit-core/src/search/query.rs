// This module is the "Search Engine" of sgit.
// It takes your natural language query and finds the most relevant commits.

use rayon::prelude::*;
use tracing::debug;

use crate::db::{CommitRecord, Store};
use crate::error::{Result, SgitError};
use crate::indexer::embed::EmbedModel;

/// This struct represents a single search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub sha: String,      // The commit SHA
    pub message: String,  // The commit message
    pub author: String,   // Who wrote it
    pub date: String,     // When it was written
    /// Similarity score from 0.0 to 1.0. 
    /// 1.0 means it's a perfect match, 0.0 means it's completely unrelated.
    pub score: f32,
}

/// Options that control how the search behaves.
pub struct SearchOptions {
    /// Maximum number of results to show (default is 3).
    pub top_n: usize,
    /// Minimum similarity score to show. We hide results that are too irrelevant.
    pub min_score: f32,
    /// Only show commits by this specific person.
    pub author_filter: Option<String>,
    /// Only show commits that happened after this date.
    pub after_date: Option<String>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            top_n: 3,
            min_score: 0.15, // Hide anything with a score below 0.15.
            author_filter: None,
            after_date: None,
        }
    }
}

/// The core search function.
/// It turns your query into a vector, loads all commits, and compares them.
pub fn search(
    query: &str,
    model: &EmbedModel,
    opts: &SearchOptions,
    repo_path: &std::path::Path,
) -> Result<Vec<SearchResult>> {
    // 1. Make sure we have an index (database) to search through.
    let store = Store::open(repo_path)?;
    let count = store.count()?;

    if count == 0 {
        return Err(SgitError::IndexNotFound);
    }

    debug!(query = %query, "Starting semantic search");

    // 2. Turn your search text into a list of numbers (embedding).
    let query_vec = model.embed_query(query)?;

    // 3. Load all indexed commits from the database.
    let commits = store.load_all()?;
    debug!(loaded = commits.len(), "Loaded commits for scoring");

    // 4. Compare your query vector against every commit vector.
    // We use 'rayon' to do this in parallel across all your CPU cores.
    let mut scored: Vec<(f32, &CommitRecord)> = commits
        .par_iter()
        .map(|commit| {
            // We use "Cosine Similarity" to measure the distance between vectors.
            let score = cosine_similarity(&query_vec, &commit.embedding);
            (score, commit)
        })
        // Filter out results that aren't relevant enough.
        .filter(|(score, _)| *score >= opts.min_score)
        .collect();

    // 5. Apply any extra filters (author or date).
    if let Some(ref author) = opts.author_filter {
        let author_lower = author.to_lowercase();
        scored.retain(|(_, c)| c.author.to_lowercase().contains(&author_lower));
    }

    if let Some(ref after) = opts.after_date {
        scored.retain(|(_, c)| c.date.as_str() >= after.as_str());
    }

    // 6. Sort the results so the most relevant ones are at the top.
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // 7. Take the top N results and return them.
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

/// Mathematical formula for Cosine Similarity.
/// It tells us how "similar" two lists of numbers are.
/// Returns 1.0 for a perfect match, and 0.0 for no similarity.
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