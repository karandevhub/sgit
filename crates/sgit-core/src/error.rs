/// All error types for sgit-core.
///
/// Using `thiserror` lets callers pattern-match on specific errors.
/// `anyhow` is used in the CLI binary; `thiserror` is used in the library.
use thiserror::Error;

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

    // ── Transparent wrappers so ? works on external types ─────────
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Git(#[from] git2::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

/// Shorthand Result type used throughout sgit-core
pub type Result<T> = std::result::Result<T, SgitError>;