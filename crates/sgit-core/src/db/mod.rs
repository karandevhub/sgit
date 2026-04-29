/// SQLite storage for git commit metadata and embeddings.
///
/// We store embeddings as BLOBs (raw f32 bytes).
/// SQLite is used because it's single-file, zero-config, and fast enough
/// for searching up to ~100k commits.
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::db_path;
use crate::error::Result;

/// A single commit record as stored in the database.
#[derive(Debug, Clone)]
pub struct CommitRecord {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub timestamp: i64,
    pub embedding: Vec<f32>,
}

pub struct Store {
    conn: Connection,
    path: PathBuf,
}

impl Store {
    /// Open the database file. Creates the table if it doesn't exist.
    pub fn open() -> Result<Self> {
        let path = db_path()?;
        let conn = Connection::open(&path)?;

        // Initialize table
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

    pub fn db_path(&self) -> &Path {
        &self.path
    }

    /// Returns the number of indexed commits.
    pub fn count(&self) -> Result<usize> {
        let count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM commits",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Returns all SHAs currently in the database.
    /// Used for incremental indexing to skip already-indexed commits.
    pub fn get_all_shas(&self) -> Result<HashSet<String>> {
        let mut stmt = self.conn.prepare("SELECT sha FROM commits")?;
        let shas = stmt.query_map([], |row| row.get(0))?
            .collect::<std::result::Result<HashSet<String>, _>>()?;
        Ok(shas)
    }

    /// Load every commit into memory for semantic scoring.
    /// (embeddings are ~1.5KB each, 10k commits = ~15MB RAM)
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

            // Convert BLOB bytes back to Vec<f32>
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

    /// Insert or update multiple commits in a single transaction.
    pub fn upsert_batch(&self, records: &[CommitRecord]) -> Result<usize> {
        // Use a transaction for massive speedup (100x+)
        let mut conn = Connection::open(&self.path)?;
        let tx = conn.transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO commits (sha, message, author, date, timestamp, embedding)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            )?;

            for r in records {
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
