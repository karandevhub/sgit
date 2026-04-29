/// sgit-core — semantic git history search library.
///
/// This crate contains all the logic: git reading, embedding, DB, search.
/// It has no CLI dependencies so it can be used as a library independently.
///
/// The binary crate (sgit) depends on this and adds the CLI shell.

pub mod config;
pub mod db;
pub mod error;
pub mod indexer;
pub mod search;

// Re-export the most commonly used types at the top level
// so external callers can use `sgit_core::SearchResult` instead of
// `sgit_core::search::query::SearchResult`
pub use error::{Result, SgitError};
pub use indexer::{run as run_index, IndexOptions, IndexStats};
pub use search::{search, SearchOptions, SearchResult};