
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tracing::{debug, info};

use crate::error::{Result, SgitError};

const MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

/// Batch size for processing.
const BATCH_SIZE: usize = 64;

/// Cache size for search queries.
const CACHE_SIZE: usize = 128;

/// AI model and query cache.
pub struct EmbedModel {
    inner: Mutex<TextEmbedding>,
    // Query embedding cache.
    cache: Mutex<LruCache<String, Vec<f32>>>,
}

impl EmbedModel {
    /// Load model from disk.
    pub fn load() -> Result<Self> {
        info!("Loading embedding model (may download ~80MB on first run)");

        let model = TextEmbedding::try_new(
            InitOptions::new(MODEL)
                .with_show_download_progress(true)
                .with_cache_dir(crate::config::model_cache_dir()?),
        )
        .map_err(|e| SgitError::ModelLoad(e.to_string()))?;

        info!("Embedding model ready");

        Ok(Self {
            inner: Mutex::new(model),
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(CACHE_SIZE).unwrap())),
        })
    }

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

    /// Embed single text.
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

pub type SharedModel = Arc<EmbedModel>;

pub fn load_shared_model() -> Result<SharedModel> {
    Ok(Arc::new(EmbedModel::load()?))
}