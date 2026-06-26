//! OpenAI-compatible embedding client for vault semantic search.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use redis::AsyncCommands;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::config::{EmbeddingsConfig, QdrantConfig, RecallConfig, RedisConfig, RerankConfig};
use crate::memory::qdrant_recall::QdrantRecall;
use crate::memory::rerank::attach_reranker;

const EMBED_CACHE_PREFIX: &str = "gzmo:embed:";
const REDIS_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub struct Embedder {
    http: Client,
    url: String,
    model: String,
    api_key: String,
    cache: Option<Arc<EmbeddingCache>>,
}

#[derive(Clone)]
struct EmbeddingCache {
    client: redis::Client,
    conn: Arc<Mutex<Option<redis::aio::ConnectionManager>>>,
    model: String,
    ttl_secs: u64,
}

#[derive(Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedData>,
}

#[derive(Deserialize)]
struct EmbedData {
    embedding: Vec<f32>,
}

impl EmbeddingCache {
    fn new(redis_cfg: &RedisConfig, embed_cfg: &EmbeddingsConfig) -> Result<Self> {
        let client = redis::Client::open(redis_cfg.url.as_str())
            .with_context(|| format!("Invalid Redis URL for embedding cache: {}", redis_cfg.url))?;
        Ok(Self {
            client,
            conn: Arc::new(Mutex::new(None)),
            model: embed_cfg.model.clone(),
            ttl_secs: embed_cfg.cache_ttl_secs,
        })
    }

    async fn conn(&self) -> Result<redis::aio::ConnectionManager> {
        if let Some(c) = self.conn.lock().await.as_ref() {
            return Ok(c.clone());
        }
        let conn = tokio::time::timeout(REDIS_CONNECT_TIMEOUT, self.client.get_connection_manager())
            .await
            .context("Embedding cache Redis connect timed out")?
            .context("Embedding cache Redis connect failed")?;
        *self.conn.lock().await = Some(conn.clone());
        Ok(conn)
    }

    fn cache_key(&self, text: &str) -> String {
        let digest = Sha256::digest(text.as_bytes());
        let hash_hex = digest.iter().map(|b| format!("{b:02x}")).collect::<String>();
        let model_key = self.model.replace(':', "_");
        format!("{EMBED_CACHE_PREFIX}{model_key}:{hash_hex}")
    }

    async fn get(&self, text: &str) -> Option<Vec<f32>> {
        let key = self.cache_key(text);
        let mut conn = match self.conn().await {
            Ok(c) => c,
            Err(e) => {
                debug!("Embedding cache unavailable on GET: {e}");
                return None;
            }
        };
        let blob: redis::RedisResult<Option<Vec<u8>>> = conn.get(&key).await;
        match blob {
            Ok(Some(bytes)) => decode_embedding(&bytes),
            Ok(None) => None,
            Err(e) => {
                debug!(error = %e, "Embedding cache GET failed");
                *self.conn.lock().await = None;
                None
            }
        }
    }

    async fn set(&self, text: &str, embedding: &[f32]) {
        if embedding.is_empty() {
            return;
        }
        let key = self.cache_key(text);
        let blob = encode_embedding(embedding);
        let mut conn = match self.conn().await {
            Ok(c) => c,
            Err(e) => {
                debug!("Embedding cache unavailable on SET: {e}");
                return;
            }
        };
        let result: redis::RedisResult<()> = conn.set_ex(key, blob, self.ttl_secs).await;
        if let Err(e) = result {
            debug!(error = %e, "Embedding cache SET failed");
            *self.conn.lock().await = None;
        }
    }
}

impl Embedder {
    pub fn from_config(cfg: &EmbeddingsConfig, redis_cfg: &RedisConfig) -> Result<Arc<Self>> {
        let url = cfg.url.trim_end_matches('/').to_string();
        let cache = if cfg.cache_enabled && redis_cfg.enabled {
            match EmbeddingCache::new(redis_cfg, cfg) {
                Ok(c) => {
                    info!(
                        url = %redis_cfg.url,
                        ttl_secs = cfg.cache_ttl_secs,
                        "Embedding Redis cache enabled"
                    );
                    Some(Arc::new(c))
                }
                Err(e) => {
                    warn!("Embedding Redis cache disabled: {e}");
                    None
                }
            }
        } else {
            None
        };
        Ok(Arc::new(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .context("Failed to build embedding HTTP client")?,
            url,
            model: cfg.model.clone(),
            api_key: cfg.api_key.clone(),
            cache,
        }))
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if let Some(cache) = &self.cache {
            if let Some(vec) = cache.get(text).await {
                debug!(dims = vec.len(), "Embedding cache hit");
                return Ok(vec);
            }
        }

        let embedding = self.embed_remote(text).await?;
        if let Some(cache) = &self.cache {
            cache.set(text, &embedding).await;
        }
        Ok(embedding)
    }

    async fn embed_remote(&self, text: &str) -> Result<Vec<f32>> {
        let endpoint = format!("{}/embeddings", self.url);
        let body = serde_json::json!({
            "model": self.model,
            "input": text,
        });

        let mut req = self.http.post(&endpoint).json(&body);
        if !self.api_key.is_empty() {
            req = req.bearer_auth(&self.api_key);
        }

        let resp = req
            .send()
            .await
            .with_context(|| format!("Embedding request failed: {endpoint}"))?
            .error_for_status()
            .context("Embedding server returned error status")?;

        let parsed: EmbedResponse = resp.json().await.context("Invalid embedding JSON")?;
        let embedding = parsed
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .context("Embedding response missing data[0]")?;

        debug!(dims = embedding.len(), "Generated embedding vector");
        Ok(embedding)
    }
}

fn encode_embedding(embedding: &[f32]) -> Vec<u8> {
    embedding
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

fn decode_embedding(blob: &[u8]) -> Option<Vec<f32>> {
    if blob.is_empty() || !blob.len().is_multiple_of(4) {
        return None;
    }
    Some(
        blob.chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect(),
    )
}

/// Open vault and attach embedder when `[embeddings].enabled`.
pub async fn open_vault_with_embeddings(
    db_path: impl AsRef<std::path::Path>,
    embed_cfg: &EmbeddingsConfig,
    redis_cfg: &RedisConfig,
    rerank_cfg: &RerankConfig,
    qdrant_cfg: &QdrantConfig,
    recall_cfg: &RecallConfig,
) -> Result<crate::memory::vault::SqliteVault> {
    let vault = crate::memory::vault::SqliteVault::open(db_path)?
        .with_recall_cfg(recall_cfg.clone());
    let vault = if !embed_cfg.enabled {
        vault
    } else {
        match Embedder::from_config(embed_cfg, redis_cfg) {
            Ok(e) => match e.embed("gzmo vault probe").await {
                Ok(v) if !v.is_empty() => {
                    info!(dims = v.len(), url = %embed_cfg.url, "Embedding server ready");
                    vault.with_embedder(Some(e))
                }
                Ok(_) => {
                    warn!("Embedding server returned empty vector — vault runs without vectors");
                    vault
                }
                Err(err) => {
                    warn!("Embedding server unreachable ({err}) — vault runs without vectors");
                    vault
                }
            },
            Err(e) => {
                warn!("Embeddings disabled — embedder init failed: {e}");
                vault
            }
        }
    };
    let vault = attach_reranker(vault, rerank_cfg).await;
    let vault = if qdrant_cfg.enabled {
        match QdrantRecall::from_config(qdrant_cfg) {
            Ok(q) => {
                info!(
                    url = %qdrant_cfg.url,
                    collection = %qdrant_cfg.collection,
                    "Qdrant recall stream enabled"
                );
                vault.with_qdrant(Some(q))
            }
            Err(e) => {
                warn!("Qdrant recall disabled: {e}");
                vault
            }
        }
    } else {
        vault
    };
    Ok(vault)
}

/// Fail fast if config requests Qdrant before a real backend exists.
pub fn assert_vault_backend(backend: &str) -> Result<()> {
    match backend.to_lowercase().as_str() {
        "sqlite" | "" => Ok(()),
        "qdrant" => anyhow::bail!(
            "vault_backend=qdrant is not implemented — use sqlite or leave vault_backend unset"
        ),
        other => anyhow::bail!("unknown vault_backend '{other}' — use sqlite"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let vec = vec![1.0_f32, -2.5, 0.0, 1024.5];
        let blob = encode_embedding(&vec);
        assert_eq!(blob.len(), 16);
        assert_eq!(decode_embedding(&blob).as_deref(), Some(vec.as_slice()));
    }

    #[test]
    fn decode_rejects_bad_length() {
        assert!(decode_embedding(&[0_u8, 1, 2]).is_none());
        assert!(decode_embedding(&[]).is_none());
    }

    #[test]
    fn cache_key_includes_model_and_text_hash() {
        let cache = EmbeddingCache {
            client: redis::Client::open("redis://127.0.0.1:6379").unwrap(),
            conn: Arc::new(Mutex::new(None)),
            model: "Qwen3-Embedding-0.6B-Q8_0.gguf".into(),
            ttl_secs: 86_400,
        };
        let k1 = cache.cache_key("hello");
        let k2 = cache.cache_key("hello");
        let k3 = cache.cache_key("world");
        assert!(k1.starts_with(EMBED_CACHE_PREFIX));
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }
}
