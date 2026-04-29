// This module handles our local AI model.
// An "embedding" is a way of turning text into a list of numbers (a vector).
// If two pieces of text have similar meanings, their vectors will be "close" 
// together in mathematical space. This is how semantic search works!

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tracing::{debug, info};

use crate::error::{Result, SgitError};

/// We use the "AllMiniLML6V2" model. 
/// It's a small but powerful model (about 80MB) that runs entirely on your CPU.
const MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

/// How many messages we process at once. Batching makes the AI much faster.
const BATCH_SIZE: usize = 64;

/// We cache the last 128 search queries so if you search for the same thing 
/// twice, it's instant.
const CACHE_SIZE: usize = 128;

/// This struct holds the actual AI model and a small cache for query results.
pub struct EmbedModel {
    // The AI model itself. We wrap it in a Mutex so multiple parts of the 
    // app can use it safely.
    inner: Mutex<TextEmbedding>,
    // A cache that remembers the embeddings for the most recent search queries.
    cache: Mutex<LruCache<String, Vec<f32>>>,
}

impl EmbedModel {
    /// Loads the model from your disk. 
    /// If it's the first time running sgit, it will download the model files (~80MB).
    pub fn load() -> Result<Self> {
        info!("Loading embedding model (may download ~80MB on first run)");

        let model = TextEmbedding::try_new(
            InitOptions::new(MODEL).with_show_download_progress(true),
        )
        .map_err(|e| SgitError::ModelLoad(e.to_string()))?;

        info!("Embedding model ready");

        Ok(Self {
            inner: Mutex::new(model),
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(CACHE_SIZE).unwrap())),
        })
    }

    /// Turns a search query into a list of numbers.
    /// It checks the cache first to see if we've already computed it.
    pub fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        {
            let mut cache = self.cache.lock();
            if let Some(cached) = cache.get(query) {
                debug!(query = %query, "Cache hit for query embedding");
                return Ok(cached.clone());
            }
        }

        debug!(query = %query, "Computing new query embedding");
        let embedding = self.embed_one(query)?;
        self.cache.lock().put(query.to_string(), embedding.clone());
        Ok(embedding)
    }

    /// Helper to embed a single piece of text.
    fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let results = self
            .inner
            .lock()
            .embed(vec![text.to_string()], None)
            .map_err(|e| SgitError::EmbedFailed(text.to_string(), e.to_string()))?;

        results.into_iter().next().ok_or_else(|| {
            SgitError::EmbedFailed(text.to_string(), "empty result from model".into())
        })
    }

    /// Processes a whole list of messages at once.
    /// This is much faster than processing them one by one.
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let mut all_embeddings = Vec::with_capacity(texts.len());

        // We split the list into small chunks (batches) to avoid using too much memory.
        for chunk in texts.chunks(BATCH_SIZE) {
            debug!(batch_size = chunk.len(), "Embedding batch");

            let embeddings = self
                .inner
                .lock()
                .embed(chunk.to_vec(), None)
                .map_err(|e| {
                    SgitError::EmbedFailed(
                        format!("batch of {} texts", chunk.len()),
                        e.to_string(),
                    )
                })?;

            all_embeddings.extend(embeddings);
        }

        Ok(all_embeddings)
    }
}

/// A thread-safe handle to our model that can be shared across the app.
pub type SharedModel = Arc<EmbedModel>;

/// Loads the model and wraps it in an Arc (Atomic Reference Counter) so 
/// it can be safely used by multiple threads at the same time.
pub fn load_shared_model() -> Result<SharedModel> {
    Ok(Arc::new(EmbedModel::load()?))
}