/// Local embedding model wrapper using fastembed 5.x.
///
/// Key facts about embeddings (important for understanding this code):
/// - An embedding converts text → a list of ~384 numbers (a "vector")
/// - Similar text produces vectors that are "close" to each other in space
/// - The model is ~80MB, downloaded once, runs fully offline on CPU
/// - fastembed's `embed()` is already batched — pass multiple texts at once
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tracing::{debug, info};

use crate::error::{Result, SgitError};

/// The embedding model we use. ALL-MiniLM-L6-v2:
/// - 384 dimensions (each text becomes 384 numbers)
/// - ~80MB download (once, then cached)
/// - Excellent quality for semantic similarity on short texts
/// - Runs in ~5ms per batch on a laptop CPU
const MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

/// How many texts to embed in one batch.
const BATCH_SIZE: usize = 64;

/// Number of query embeddings to cache in memory.
const CACHE_SIZE: usize = 128;

/// A loaded, ready-to-use embedding model with an LRU cache.
pub struct EmbedModel {
    /// fastembed 5.x requires &mut self for embed(), so wrap in Mutex for &self usage
    inner: Mutex<TextEmbedding>,
    /// Cache: query text → embedding vector
    cache: Mutex<LruCache<String, Vec<f32>>>,
}

impl EmbedModel {
    /// Load the model from disk (or download it on first run).
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

    /// Embed a single query string.
    /// Checks the LRU cache first — if the query was seen recently, returns instantly.
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

    /// Embed a single text string.
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

    /// Embed a batch of texts. Much faster than one-by-one.
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let mut all_embeddings = Vec::with_capacity(texts.len());

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

/// Shared, thread-safe handle to an EmbedModel.
pub type SharedModel = Arc<EmbedModel>;

/// Create a new shared model handle.
pub fn load_shared_model() -> Result<SharedModel> {
    Ok(Arc::new(EmbedModel::load()?))
}