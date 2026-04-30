// This module handles where sgit saves its data on your computer.
// Every Operating System (Mac, Linux, Windows) has a specific place 
// where apps are supposed to save their files. This code ensures we 
// follow those rules so sgit works perfectly everywhere.

use directories::ProjectDirs;
use std::path::PathBuf;

use crate::error::{Result, SgitError};

/// Finds the main folder where sgit will store its database and AI models.
/// It also creates the folder if it doesn't exist yet.
pub fn data_dir() -> Result<PathBuf> {
    // ProjectDirs handles the complex task of finding the right folder for each OS.
    let proj = ProjectDirs::from("ai", "sgit", "sgit")
        .ok_or_else(|| {
            SgitError::DataDirCreate(
                "unknown".into(),
                "Could not determine home directory".into(),
            )
        })?;

    let dir = proj.data_dir().to_path_buf();
    
    // Create the directory if it's not already there.
    std::fs::create_dir_all(&dir)
        .map_err(|e| SgitError::DataDirCreate(dir.display().to_string(), e.to_string()))?;

    Ok(dir)
}

/// Returns the full path to the 'index.db' file for a specific repository.
/// The database is stored inside the hidden '.git/sgit' folder so it follows 
/// the project if it's moved, but is never tracked by Git.
pub fn db_path(repo_path: &std::path::Path) -> Result<PathBuf> {
    let repo = git2::Repository::discover(repo_path).map_err(|e| {
        SgitError::NoRepository(format!("{}: {}", repo_path.display(), e))
    })?;

    let git_dir = repo.path(); // This is the path to the .git folder
    let sgit_dir = git_dir.join("sgit");

    // Create the .git/sgit folder if it doesn't exist
    if !sgit_dir.exists() {
        std::fs::create_dir_all(&sgit_dir)
            .map_err(|e| SgitError::DataDirCreate(sgit_dir.display().to_string(), e.to_string()))?;
    }

    Ok(sgit_dir.join("index.db"))
}

/// Returns the folder where the AI model files will be downloaded.
pub fn model_cache_dir() -> Result<PathBuf> {
    let dir = data_dir()?.join("models");
    std::fs::create_dir_all(&dir)
        .map_err(|e| SgitError::DataDirCreate(dir.display().to_string(), e.to_string()))?;
    Ok(dir)
}

/// This function helps us print a path in the terminal in a way that 
/// looks nice and is easy to read.
pub fn display_path(p: &std::path::Path) -> String {
    p.display().to_string()
}