// This is the "Brain" of the sgit application. 
// It contains all the complex logic for reading Git history, running 
// the AI model, managing the database, and performing semantic searches.
//
// The reason this is a separate "crate" (library) is so that the logic 
// can be re-used in different ways (like a GUI or a web server) without 
// being tied to the command-line interface.

pub mod config;
pub mod db;
pub mod error;
pub mod indexer;
pub mod search;

// We "re-export" the most important parts of sgit-core here so that 
// other crates (like our CLI binary) can access them easily.
pub use error::{Result, SgitError};
pub use indexer::{run as run_index, IndexOptions, IndexStats};
pub use search::{search, SearchOptions, SearchResult};