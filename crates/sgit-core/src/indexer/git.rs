/// Reads git commit history from the current directory using libgit2.
///
/// We use libgit2 (via the `git2` crate) instead of shelling out to `git`
/// because: no process spawning overhead, works even if git isn't installed,
/// and gives us structured access to commit objects.
use chrono::{FixedOffset, TimeZone};
use git2::{Repository, Sort};
use tracing::{debug, info, warn};

use crate::error::{Result, SgitError};

/// A single commit record pulled from git history.
#[derive(Debug, Clone)]
pub struct GitCommit {
    /// Short 8-char SHA, e.g. "a1b2c3d4"
    pub sha: String,
    /// Full commit message (summary line only, not body)
    pub message: String,
    /// Author display name
    pub author: String,
    /// ISO 8601 date string, e.g. "2024-03-15"
    pub date: String,
    /// Unix timestamp — used for sorting
    pub timestamp: i64,
}

/// Walk the entire git history of the repo at `repo_path` and return all commits.
///
/// Commits with useless messages ("wip", "fix", ".", single words) are filtered out
/// because they produce bad embeddings and pollute search results.
pub fn read_commits(repo_path: &std::path::Path) -> Result<Vec<GitCommit>> {
    // Open the repository — looks for .git/ starting at repo_path
    let repo = Repository::discover(repo_path).map_err(|e| {
        SgitError::NoRepository(format!("{}: {}", repo_path.display(), e))
    })?;

    info!(path = %repo_path.display(), "Opened git repository");

    let mut revwalk = repo.revwalk()?;

    // Sort by time, newest first (matches `git log` default behaviour)
    revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
    revwalk.push_head()?;

    let mut commits = Vec::new();
    let mut skipped = 0usize;

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        // Get commit message — skip if missing or useless
        let raw_message = match commit.summary() {
            Some(m) => m.to_string(),
            None => {
                skipped += 1;
                continue;
            }
        };

        if is_useless_message(&raw_message) {
            debug!(sha = %oid, msg = %raw_message, "Skipping low-quality commit message");
            skipped += 1;
            continue;
        }

        let author = commit
            .author()
            .name()
            .unwrap_or("Unknown Author")
            .to_string();

        let timestamp = commit.time().seconds();
        let date = format_commit_date(timestamp, commit.time().offset_minutes());

        commits.push(GitCommit {
            sha: oid.to_string()[..8].to_string(),
            message: raw_message,
            author,
            date,
            timestamp,
        });
    }

    info!(
        total = commits.len(),
        skipped = skipped,
        "Finished reading git history"
    );

    Ok(commits)
}

/// Format a git commit timestamp into a human-readable date.
/// Uses the commit's local timezone offset (same as `git log` shows).
fn format_commit_date(unix_secs: i64, offset_minutes: i32) -> String {
    let offset = FixedOffset::east_opt(offset_minutes * 60)
        .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());

    match offset.timestamp_opt(unix_secs, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d").to_string(),
        _ => {
            // Fallback if timestamp is out of range
            warn!(unix_secs, "Could not parse commit timestamp");
            "unknown".to_string()
        }
    }
}

/// Returns true for commit messages that are too short or generic to be useful.
/// These produce low-quality embeddings that pollute search results.
fn is_useless_message(msg: &str) -> bool {
    let trimmed = msg.trim();

    // Too short
    if trimmed.len() < 5 {
        return true;
    }

    // Only one word (e.g. "fix", "wip", "update", ".")
    if trimmed.split_whitespace().count() < 2 {
        return true;
    }

    // Common garbage commit messages
    let garbage = [
        "wip", "fix", "fixes", "update", "updates", ".", "..", "temp",
        "test", "testing", "misc", "stuff", "changes", "work in progress",
        "minor changes", "minor fix", "minor update", "cleanup",
    ];
    if garbage.contains(&trimmed.to_lowercase().as_str()) {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_useless_messages() {
        assert!(is_useless_message("wip"));
        assert!(is_useless_message("fix"));
        assert!(is_useless_message("."));
        assert!(!is_useless_message("fix authentication timeout bug"));
        assert!(!is_useless_message("refactor: extract DB connection pool"));
    }

    #[test]
    fn test_format_date() {
        // Unix timestamp 0 = 1970-01-01 UTC
        let date = format_commit_date(0, 0);
        assert_eq!(date, "1970-01-01");
    }
}