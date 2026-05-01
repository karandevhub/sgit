
use directories::ProjectDirs;
use std::path::PathBuf;

use crate::error::{Result, SgitError};

/// Get sgit data directory.
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

pub fn db_path(repo_path: &std::path::Path) -> Result<PathBuf> {
    let repo = git2::Repository::discover(repo_path).map_err(|e| {
        SgitError::NoRepository(format!("{}: {}", repo_path.display(), e))
    })?;

    let git_dir = repo.path();
    let sgit_dir = git_dir.join("sgit");

    if !sgit_dir.exists() {
        std::fs::create_dir_all(&sgit_dir)
            .map_err(|e| SgitError::DataDirCreate(sgit_dir.display().to_string(), e.to_string()))?;
    }

    Ok(sgit_dir.join("index.db"))
}

pub fn model_cache_dir() -> Result<PathBuf> {
    let dir = data_dir()?.join("models");
    std::fs::create_dir_all(&dir)
        .map_err(|e| SgitError::DataDirCreate(dir.display().to_string(), e.to_string()))?;
    Ok(dir)
}

pub fn display_path(p: &std::path::Path) -> String {
    p.display().to_string()
}