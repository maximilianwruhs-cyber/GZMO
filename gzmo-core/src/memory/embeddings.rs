//! OpenAI-compatible embedding client for vault semantic search.

use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::config::{EmbeddingsConfig, QdrantConfig, RerankConfig};
use crate::memory::qdrant_recall::QdrantRecall;
use crate::memory::rerank::attach_reranker;

#[derive(Clone)]
pub struct Embedder {
    http: Client,
    url: String,
    model: String,
    api_key: String,
}

#[derive(Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedData>,
}

#[derive(Deserialize)]
struct EmbedData {
    embedding: Vec<f32>,
}

impl Embedder {
    pub fn from_config(cfg: &EmbeddingsConfig) -> Result<Arc<Self>> {
        let url = cfg.url.trim_end_matches('/').to_string();
        Ok(Arc::new(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .context("Failed to build embedding HTTP client")?,
            url,
            model: cfg.model.clone(),
            api_key: cfg.api_key.clone(),
        }))
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
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

/// Open vault and attach embedder when `[embeddings].enabled`.
pub async fn open_vault_with_embeddings(
    db_path: impl AsRef<std::path::Path>,
    embed_cfg: &EmbeddingsConfig,
    rerank_cfg: &RerankConfig,
    qdrant_cfg: &QdrantConfig,
) -> Result<crate::memory::vault::SqliteVault> {
    let vault = crate::memory::vault::SqliteVault::open(db_path)?;
    let vault = if !embed_cfg.enabled {
        vault
    } else {
        match Embedder::from_config(embed_cfg) {
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
