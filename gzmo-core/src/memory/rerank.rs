//! OpenAI/Jina-compatible rerank client (`POST /v1/rerank`).

use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::config::RerankConfig;

#[derive(Clone)]
pub struct Reranker {
    http: Client,
    url: String,
    model: String,
    api_key: String,
    prefetch_multiplier: usize,
}

#[derive(Deserialize)]
struct RerankResponse {
    results: Vec<RerankHit>,
}

#[derive(Deserialize)]
struct RerankHit {
    index: usize,
    #[serde(alias = "score", default)]
    relevance_score: f64,
}

impl Reranker {
    pub fn from_config(cfg: &RerankConfig) -> Result<Arc<Self>> {
        let url = cfg.url.trim_end_matches('/').to_string();
        Ok(Arc::new(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .context("Failed to build rerank HTTP client")?,
            url,
            model: cfg.model.clone(),
            api_key: cfg.api_key.clone(),
            prefetch_multiplier: cfg.prefetch_multiplier.max(1),
        }))
    }

    /// Rerank `documents` against `query`; returns `(doc_index, score)` sorted best-first.
    pub async fn rerank(
        &self,
        query: &str,
        documents: &[String],
        top_n: Option<usize>,
    ) -> Result<Vec<(usize, f64)>> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }

        let endpoint = format!("{}/rerank", self.url);
        let mut body = serde_json::json!({
            "model": self.model,
            "query": query,
            "documents": documents,
        });
        if let Some(n) = top_n {
            body["top_n"] = serde_json::json!(n);
        }

        let mut req = self.http.post(&endpoint).json(&body);
        if !self.api_key.is_empty() {
            req = req.bearer_auth(&self.api_key);
        }

        let resp = req
            .send()
            .await
            .with_context(|| format!("Rerank request failed: {endpoint}"))?
            .error_for_status()
            .context("Rerank server returned error status")?;

        let parsed: RerankResponse = resp.json().await.context("Invalid rerank JSON")?;
        let mut out: Vec<(usize, f64)> = parsed
            .results
            .into_iter()
            .map(|h| (h.index, h.relevance_score))
            .collect();
        out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        debug!(hits = out.len(), "Reranked document batch");
        Ok(out)
    }

    pub fn prefetch_limit(&self, limit: usize) -> usize {
        (limit * self.prefetch_multiplier).clamp(limit, 50)
    }
}

/// Attach reranker when enabled; logs and continues without on failure.
pub async fn attach_reranker(
    vault: crate::memory::vault::SqliteVault,
    cfg: &RerankConfig,
) -> crate::memory::vault::SqliteVault {
    if !cfg.enabled {
        return vault;
    }
    match Reranker::from_config(cfg) {
        Ok(r) => match r
            .rerank("vault probe", &["semantic memory".to_string()], Some(1))
            .await
        {
            Ok(_) => {
                tracing::info!(url = %cfg.url, "Rerank server ready");
                vault.with_reranker(Some(r))
            }
            Err(err) => {
                warn!("Rerank server unreachable ({err}) — vault search without rerank");
                vault
            }
        },
        Err(e) => {
            warn!("Rerank disabled — init failed: {e}");
            vault
        }
    }
}
