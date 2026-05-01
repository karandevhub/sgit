
pub mod config;
pub mod db;
pub mod error;
pub mod indexer;
pub mod search;

pub use error::{Result, SgitError};
pub use indexer::{run as run_index, IndexOptions, IndexStats};
pub use search::{search, SearchOptions, SearchResult};