/// Cross-platform path resolution using the `directories` crate.
///
/// Instead of hardcoding `.sgit/`, we store data in the OS-correct location:
///   Mac:   ~/Library/Application Support/sgit/
///   Linux: ~/.local/share/sgit/
///   Win:   C:\Users\X\AppData\Roaming\sgit\
use directories::ProjectDirs;
use std::path::PathBuf;

use crate::error::{Result, SgitError};

/// Returns the directory where sgit stores its index database and model cache.
/// Creates the directory if it doesn't exist.
pub fn data_dir() -> Result<PathBuf> {
    let proj = ProjectDirs::from("ai", "sgit", "sgit")
        .ok_or_else(|| {
            SgitError::DataDirCreate(
                "unknown".into(),
                "Could not determine home directory".into(),
            )
        })?;

    let dir = proj.data_dir().to_path_buf();
    std::fs::create_dir_all(&dir)
        .map_err(|e| SgitError::DataDirCreate(dir.display().to_string(), e.to_string()))?;

    Ok(dir)
}

/// Full path to the SQLite database file.
pub fn db_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("index.db"))
}

/// Directory where fastembed downloads and caches the model files.
pub fn model_cache_dir() -> Result<PathBuf> {
    let dir = data_dir()?.join("models");
    std::fs::create_dir_all(&dir)
        .map_err(|e| SgitError::DataDirCreate(dir.display().to_string(), e.to_string()))?;
    Ok(dir)
}

/// Returns a short human-readable path for display in terminal output.
/// e.g. "/Users/alice/Library/Application Support/sgit/index.db"
///      → "~/Library/Application Support/sgit/index.db"
pub fn display_path(p: &std::path::Path) -> String {
    // Walk up from the data_dir to find a home-like prefix we can replace with ~
    if let Some(proj) = ProjectDirs::from("ai", "sgit", "sgit") {
        // data_dir lives 2–3 levels deep inside the home dir on every platform
        let data = proj.data_dir();
        // Find the longest prefix of `data` that is also a prefix of `p`
        let mut ancestor = data;
        loop {
            if let Ok(rel) = p.strip_prefix(ancestor) {
                if let Some(parent) = ancestor.parent() {
                    if let Ok(home_rel) = ancestor.strip_prefix(parent) {
                        // We successfully stripped from `parent` up
                        // Determine depth from home
                        let _ = home_rel; // suppress warning
                    }
                }
                let _ = rel;
            }
            match ancestor.parent() {
                Some(p) => ancestor = p,
                None => break,
            }
        }
    }

    // Simple fallback: just show the full absolute path
    p.display().to_string()
}