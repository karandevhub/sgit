// This module handles the SQLite database where we store our git commit data.
// We use SQLite because it's a simple, single-file database that doesn't 
// require any setup from the user. It's perfect for a CLI tool like this!

use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::db_path;
use crate::error::Result;

/// This struct represents a single commit as it's saved in our database.
/// It holds all the basic info like the SHA (id), message, author, and the 
/// mathematical "embedding" vector that allows us to do semantic search.
#[derive(Debug, Clone)]
pub struct CommitRecord {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub timestamp: i64,
    pub embedding: Vec<f32>, // This is the "meaning" of the commit message as a list of numbers.
}

/// The Store struct is our handle to the database.
/// It wraps a SQLite connection and keeps track of where the file is.
pub struct Store {
    conn: Connection,
    path: PathBuf,
}

impl Store {
    /// Opens the database file for a specific repository.
    /// If the file doesn't exist, it creates it and sets up the table.
    pub fn open(repo_path: &Path) -> Result<Self> {
        let path = db_path(repo_path)?;
        let conn = Connection::open(&path)?;

        // We create a table called 'commits' to store our data.
        // The 'sha' is the unique ID for each commit.
        // The 'embedding' is stored as a BLOB (Binary Large Object) because it's just raw bytes.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS commits (
                sha TEXT PRIMARY KEY,
                message TEXT NOT NULL,
                author TEXT NOT NULL,
                date TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                embedding BLOB NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn, path })
    }

    /// Returns the absolute path to where the database file is stored on your disk.
    pub fn db_path(&self) -> &Path {
        &self.path
    }

    /// Returns how many commits have been indexed so far.
    pub fn count(&self) -> Result<usize> {
        let count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM commits",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Gets a list of all commit SHAs we've already indexed.
    /// This is used for "incremental" indexing so we don't re-process 
    /// commits we already have in the database.
    pub fn get_all_shas(&self) -> Result<HashSet<String>> {
        let mut stmt = self.conn.prepare("SELECT sha FROM commits")?;
        let shas = stmt.query_map([], |row| row.get(0))?
            .collect::<std::result::Result<HashSet<String>, _>>()?;
        Ok(shas)
    }

    /// Loads every single commit from the database into memory.
    /// We do this during search so we can compare the user's query 
    /// against every commit very quickly using your CPU's cores.
    pub fn load_all(&self) -> Result<Vec<CommitRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT sha, message, author, date, timestamp, embedding FROM commits"
        )?;

        let records = stmt.query_map([], |row| {
            let sha: String = row.get(0)?;
            let message: String = row.get(1)?;
            let author: String = row.get(2)?;
            let date: String = row.get(3)?;
            let timestamp: i64 = row.get(4)?;
            let embedding_bytes: Vec<u8> = row.get(5)?;

            // The database stores the embedding as raw bytes, so we 
            // convert them back into a list of numbers (Vec<f32>).
            let embedding = bytes_to_f32(&embedding_bytes);

            Ok(CommitRecord {
                sha,
                message,
                author,
                date,
                timestamp,
                embedding,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// Saves a batch of commits to the database all at once.
    /// We use a "transaction" here because it's much faster than 
    /// saving them one-by-one.
    pub fn upsert_batch(&self, records: &[CommitRecord]) -> Result<usize> {
        let mut conn = Connection::open(&self.path)?;
        let tx = conn.transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO commits (sha, message, author, date, timestamp, embedding)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            )?;

            for r in records {
                // Convert our list of numbers into raw bytes for storage.
                let embedding_bytes = f32_to_bytes(&r.embedding);
                stmt.execute(params![
                    r.sha,
                    r.message,
                    r.author,
                    r.date,
                    r.timestamp,
                    embedding_bytes
                ])?;
            }
        }

        tx.commit()?;
        Ok(records.len())
    }
}

// ── Helpers ───────────────────────────────────────────────────────
// These functions help us convert our AI data (f32 numbers) into 
// a format that SQLite can save on disk (bytes), and back again.

fn f32_to_bytes(floats: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(floats.len() * 4);
    for &f in floats {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

fn bytes_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
        .collect()
}
