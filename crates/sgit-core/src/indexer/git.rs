// This module is responsible for reading your Git history.
// We use a library called 'libgit2' which allows us to read commits 
// directly from the .git folder without needing the 'git' command installed.

use chrono::{FixedOffset, TimeZone};
use git2::{Repository, Sort};
use tracing::{debug, info, warn};

use crate::error::{Result, SgitError};

/// This struct holds the basic information for a single commit.
#[derive(Debug, Clone)]
pub struct GitCommit {
    pub sha: String,      // The short unique ID of the commit (e.g., "a1b2c3d4")
    pub message: String,  // The text the developer wrote for this commit
    pub author: String,   // Who wrote the commit
    pub date: String,     // When it was written (formatted like YYYY-MM-DD)
    pub timestamp: i64,   // The raw time number (used for sorting)
}

/// Walks through your entire Git history and returns a list of all commits.
/// We filter out "useless" messages like "wip" or "." because they don't 
/// have enough meaning for the AI to understand them.
pub fn read_commits(repo_path: &std::path::Path) -> Result<Vec<GitCommit>> {
    // Try to find the .git folder in the given path.
    let repo = Repository::discover(repo_path).map_err(|e| {
        SgitError::NoRepository(format!("{}: {}", repo_path.display(), e))
    })?;

    info!(path = %repo_path.display(), "Opened git repository");

    let mut revwalk = repo.revwalk()?;

    // We sort the commits by time, so the newest ones come first.
    revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
    revwalk.push_head()?;

    let mut commits = Vec::new();
    let mut skipped = 0usize;

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        // We only care about the first line of the commit message (the summary).
        let raw_message = match commit.summary() {
            Some(m) => m.to_string(),
            None => {
                skipped += 1;
                continue;
            }
        };

        // Skip messages that don't have enough information (like "fix" or "update").
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
        // Convert the raw Git time into a nice human-readable string.
        let date = format_commit_date(timestamp, commit.time().offset_minutes());

        commits.push(GitCommit {
            sha: oid.to_string()[..8].to_string(), // Just take the first 8 characters of the SHA.
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

/// Helper function to turn a raw timestamp into a "YYYY-MM-DD" string.
fn format_commit_date(unix_secs: i64, offset_minutes: i32) -> String {
    let offset = FixedOffset::east_opt(offset_minutes * 60)
        .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());

    match offset.timestamp_opt(unix_secs, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d").to_string(),
        _ => {
            warn!(unix_secs, "Could not parse commit timestamp");
            "unknown".to_string()
        }
    }
}

/// This function helps us avoid cluttering our search index with messages 
/// that don't mean anything (like "...", "temp", or single words).
fn is_useless_message(msg: &str) -> bool {
    let trimmed = msg.trim();

    // If it's shorter than 5 characters, it's probably not useful.
    if trimmed.len() < 5 {
        return true;
    }

    // If it's only one word, the AI won't have enough context to search it well.
    if trimmed.split_whitespace().count() < 2 {
        return true;
    }

    // A list of common "lazy" commit messages we want to ignore.
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
        let date = format_commit_date(0, 0);
        assert_eq!(date, "1970-01-01");
    }
}