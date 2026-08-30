//! Qdrant vector search for honeypot recall (RRF stream B′).

use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tracing::debug;
use uuid::Uuid;

use crate::config::QdrantConfig;

/// Upper bound clamped onto every caller-supplied `limit` search parameter.
pub const MAX_SEARCH_LIMIT: usize = 100;

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
        let Some(body) = Self::build_search_body(vector, limit, with_payload) else {
            return Ok(Vec::new());
        };
        let url = format!(
            "{}/collections/{}/points/search",
            self.base_url, self.collection
        );
        // Do not add a Qdrant payload filter on `is_latest` here. Living points
        // predate the stamp; filtering now would empty the vector stream until a
        // full re-sync. SQLite `take_assertable_prefetch` is the current-time gate.
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

    fn build_search_body(
        vector: &[f32],
        limit: usize,
        with_payload: bool,
    ) -> Option<serde_json::Value> {
        if vector.is_empty() {
            return None;
        }
        Some(serde_json::json!({
            "vector": vector,
            "limit": limit.clamp(1, MAX_SEARCH_LIMIT),
            "with_payload": with_payload,
        }))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_build_search_body_clamps_limit() {
        let vec = vec![0.1, 0.2];

        let body_zero = QdrantRecall::build_search_body(&vec, 0, false).unwrap();
        assert_eq!(body_zero["limit"], 1);

        let body_normal = QdrantRecall::build_search_body(&vec, 10, false).unwrap();
        assert_eq!(body_normal["limit"], 10);

        let body_max = QdrantRecall::build_search_body(&vec, MAX_SEARCH_LIMIT + 50, false).unwrap();
        assert_eq!(body_max["limit"], MAX_SEARCH_LIMIT);
    }

    #[test]
    fn test_build_search_body_empty_vector() {
        assert!(QdrantRecall::build_search_body(&[], 10, false).is_none());
    }

    #[test]
    fn test_parse_point_id() {
        let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
        let expected = Uuid::parse_str(uuid_str).unwrap();

        assert_eq!(parse_point_id(&json!(uuid_str)), Some(expected));
        assert_eq!(parse_point_id(&json!({"uuid": uuid_str})), Some(expected));
        assert_eq!(parse_point_id(&json!({"id": uuid_str})), None);
        assert_eq!(parse_point_id(&json!(42)), None);
    }
}
