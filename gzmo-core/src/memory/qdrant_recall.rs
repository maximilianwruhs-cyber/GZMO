//! Qdrant vector search for honeypot recall (RRF stream B′).

use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tracing::debug;
use uuid::Uuid;

use crate::config::QdrantConfig;

#[derive(Clone)]
pub struct QdrantRecall {
    http: Client,
    base_url: String,
    collection: String,
}

#[derive(Deserialize)]
struct SearchResponse {
    result: Vec<SearchHit>,
}

#[derive(Deserialize)]
struct SearchHit {
    id: serde_json::Value,
    score: Option<f64>,
}

impl QdrantRecall {
    pub fn from_config(cfg: &QdrantConfig) -> Result<Arc<Self>> {
        let base_url = cfg.url.trim_end_matches('/').to_string();
        Ok(Arc::new(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .context("Qdrant HTTP client")?,
            base_url,
            collection: cfg.collection.clone(),
        }))
    }

    /// Vector search; returns honeypot fact UUIDs best-first.
    pub async fn search_ids(&self, vector: &[f32], limit: usize) -> Result<Vec<Uuid>> {
        if vector.is_empty() {
            return Ok(Vec::new());
        }
        let url = format!(
            "{}/collections/{}/points/search",
            self.base_url, self.collection
        );
        let body = serde_json::json!({
            "vector": vector,
            "limit": limit,
            "with_payload": false,
        });
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Qdrant search failed: {url}"))?
            .error_for_status()
            .context("Qdrant search HTTP error")?;

        let parsed: SearchResponse = resp.json().await.context("Qdrant search JSON")?;
        let mut ids = Vec::with_capacity(parsed.result.len());
        for hit in parsed.result {
            if let Some(id) = parse_point_id(&hit.id) {
                ids.push(id);
            }
        }
        debug!(count = ids.len(), collection = %self.collection, "Qdrant recall stream");
        Ok(ids)
    }
}

fn parse_point_id(value: &serde_json::Value) -> Option<Uuid> {
    match value {
        serde_json::Value::String(s) => Uuid::parse_str(s).ok(),
        serde_json::Value::Object(o) => o
            .get("uuid")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok()),
        _ => None,
    }
}
