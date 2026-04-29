// This module defines all the things that can go wrong in sgit.
// By centralizing errors here, we can provide clear and helpful messages 
// to the user when something doesn't work as expected.

use thiserror::Error;

/// The SgitError enum lists every possible error that can happen in the 
/// core logic of sgit.
#[derive(Debug, Error)]
pub enum SgitError {
    // ── Setup errors ──────────────────────────────────────────────
    #[error("No .git directory found in '{0}'. Is this a git repository?")]
    NoRepository(String),

    #[error("Could not create data directory at '{0}': {1}")]
    DataDirCreate(String, String),

    // ── Index errors ──────────────────────────────────────────────
    #[error("Index not found. Run `sgit index` first.")]
    IndexNotFound,

    #[error("Failed to read git history: {0}")]
    GitRead(String),

    // ── Embedding errors ───────────────────────────────────────────
    #[error("Embedding model failed to load: {0}")]
    ModelLoad(String),

    #[error("Embedding failed for text '{0}': {1}")]
    EmbedFailed(String, String),

    // ── Database errors ────────────────────────────────────────────
    #[error("Database error: {0}")]
    Database(String),

    // ── Search errors ─────────────────────────────────────────────
    #[error("No commits match your query. Try broader terms.")]
    NoResults,

    // ── Wrapper errors ─────────────────────────────────────────────
    // These allow sgit to automatically convert errors from other 
    // libraries (like Git or SQLite) into our own SgitError format.

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Git(#[from] git2::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

/// A convenient shorthand for Result<T, SgitError>.
pub type Result<T> = std::result::Result<T, SgitError>;