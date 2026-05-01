
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SgitError {
    #[error("No .git directory found in '{0}'. Is this a git repository?")]
    NoRepository(String),

    #[error("Could not create data directory at '{0}': {1}")]
    DataDirCreate(String, String),

    #[error("Index not found. Run `sgit index` first.")]
    IndexNotFound,

    #[error("Failed to read git history: {0}")]
    GitRead(String),

    #[error("Embedding model failed to load: {0}")]
    ModelLoad(String),

    #[error("Embedding failed for text '{0}': {1}")]
    EmbedFailed(String, String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("No commits match your query. Try broader terms.")]
    NoResults,


    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Git(#[from] git2::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, SgitError>;