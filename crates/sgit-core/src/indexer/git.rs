
use chrono::{FixedOffset, TimeZone};
use git2::{Repository, Sort};
use tracing::{debug, info, warn};

use crate::error::{Result, SgitError};

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub timestamp: i64,
}

pub fn read_commits(repo_path: &std::path::Path) -> Result<Vec<GitCommit>> {
    let repo = Repository::discover(repo_path).map_err(|e| {
        SgitError::NoRepository(format!("{}: {}", repo_path.display(), e))
    })?;

    info!(path = %repo_path.display(), "Opened git repository");

    let mut revwalk = repo.revwalk()?;

    revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
    revwalk.push_head()?;

    let mut commits = Vec::new();
    let mut skipped = 0usize;

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

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

fn is_useless_message(msg: &str) -> bool {
    let trimmed = msg.trim();

    if trimmed.len() < 5 {
        return true;
    }
    if trimmed.split_whitespace().count() < 2 {
        return true;
    }

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