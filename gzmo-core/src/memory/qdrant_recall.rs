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

#[derive(Debug, Clone)]
pub struct QdrantPayloadHit {
    pub score: f64,
    pub payload: serde_json::Value,
}

#[derive(Deserialize)]
struct SearchHit {
    id: serde_json::Value,
    score: Option<f64>,
    #[serde(default)]
    payload: Option<serde_json::Value>,
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

    pub fn with_collection(&self, collection: impl Into<String>) -> Self {
        let mut s = self.clone();
        s.collection = collection.into();
        s
    }

    /// Vector search; returns honeypot fact UUIDs best-first.
    pub async fn search_ids(&self, vector: &[f32], limit: usize) -> Result<Vec<Uuid>> {
        Ok(self
            .search_hits(vector, limit, false)
            .await?
            .into_iter()
            .filter_map(|h| parse_point_id(&h.id))
            .collect())
    }

    /// Vector search with optional payload (Pi knowledge collection).
    pub async fn search_with_payload(
        &self,
        vector: &[f32],
        limit: usize,
    ) -> Result<Vec<QdrantPayloadHit>> {
        let hits = self.search_hits(vector, limit, true).await?;
        Ok(hits
            .into_iter()
            .map(|h| QdrantPayloadHit {
                score: h.score.unwrap_or(0.0),
                payload: h.payload.unwrap_or(serde_json::Value::Null),
            })
            .collect())
    }

    async fn search_hits(
        &self,
        vector: &[f32],
        limit: usize,
        with_payload: bool,
    ) -> Result<Vec<SearchHit>> {
        if vector.is_empty() {
            return Ok(Vec::new());
        }
        let url = format!(
            "{}/collections/{}/points/search",
            self.base_url, self.collection
        );
        // Do not add a Qdrant payload filter on `is_latest` here. Living points
        // predate the stamp; filtering now would empty the vector stream until a
        // full re-sync. SQLite `take_assertable_prefetch` is the current-time gate.
        let body = serde_json::json!({
            "vector": vector,
            "limit": limit,
            "with_payload": with_payload,
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
        debug!(
            count = parsed.result.len(),
            collection = %self.collection,
            "Qdrant recall stream"
        );
        Ok(parsed.result)
    }

    /// Upsert a single point (vector + payload) into the configured
    /// collection. Idempotent: calling this again with the same `id`
    /// replaces the vector/payload rather than duplicating the point.
    pub async fn upsert_point(
        &self,
        id: Uuid,
        vector: &[f32],
        payload: serde_json::Value,
    ) -> Result<()> {
        let url = format!(
            "{}/collections/{}/points?wait=true",
            self.base_url, self.collection
        );
        let body = serde_json::json!({
            "points": [{
                "id": id.to_string(),
                "vector": vector,
                "payload": payload,
            }]
        });
        self.http
            .put(&url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Qdrant upsert failed: {url}"))?
            .error_for_status()
            .context("Qdrant upsert HTTP error")?;
        Ok(())
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
