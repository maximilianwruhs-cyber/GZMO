/// Spark-to-distill session correlation bridge.
///
/// Spark events use `anchor_id` (not Pi session_id). The bridge indexes recent
/// spark_complete events by time and injects lineage into distill_complete metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Represents a spark event extracted from the Synapse bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkEvent {
    pub event_type: String,
    pub id: String,
    pub source: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

/// On-disk cache for distill consumers (`data/spark-distill-bridge-cache.json`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SparkDistillCache {
    pub updated_at: Option<String>,
    pub sparks: Vec<SparkEvent>,
}

/// Configuration for the spark-to-distill correlation bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkDistillBridgeConfig {
    pub enabled: bool,
    pub lookback_minutes: u64,
    pub inject_spark_lineage: bool,
    pub max_cached_sparks: usize,
}

impl Default for SparkDistillBridgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            lookback_minutes: 30,
            inject_spark_lineage: true,
            max_cached_sparks: 200,
        }
    }
}

/// Bridge state — tracks recent spark events for time-window correlation.
pub struct SparkDistillBridge {
    config: SparkDistillBridgeConfig,
    /// Legacy session_id index (often empty — sparks lack session_id).
    spark_cache: HashMap<String, Vec<SparkEvent>>,
    /// Chronological spark tail for lookback correlation.
    recent_sparks: Vec<SparkEvent>,
}

impl SparkDistillBridge {
    pub fn new(config: SparkDistillBridgeConfig) -> Self {
        Self {
            config,
            spark_cache: HashMap::new(),
            recent_sparks: Vec::new(),
        }
    }

    fn push_recent(&mut self, event: SparkEvent) {
        self.recent_sparks.push(event);
        let max = self.config.max_cached_sparks.max(20);
        if self.recent_sparks.len() > max {
            let drop = self.recent_sparks.len() - max;
            self.recent_sparks.drain(0..drop);
        }
    }

    /// Register a spark event for later correlation with distill sessions.
    pub fn register_spark(&mut self, event: SparkEvent) {
        if !self.config.enabled {
            return;
        }
        if let Some(session_id) = event.data.get("session_id").and_then(|s| s.as_str()) {
            self.spark_cache
                .entry(session_id.to_string())
                .or_default()
                .push(event.clone());
        }
        self.push_recent(event);
    }

    /// Given a distill session, return correlated spark events (session_id match).
    pub fn correlate_sparks(&self, session_id: &str) -> Vec<&SparkEvent> {
        if !self.config.enabled {
            return Vec::new();
        }
        self.spark_cache
            .get(session_id)
            .map(|events| events.iter().collect())
            .unwrap_or_default()
    }

    pub fn has_spark_lineage(&self, session_id: &str) -> bool {
        self.spark_cache
            .get(session_id)
            .map(|events| !events.is_empty())
            .unwrap_or(false)
    }

    pub fn spark_count(&self) -> usize {
        self.recent_sparks.len()
    }

    /// Sparks within lookback window (time-based — primary correlation path).
    pub fn sparks_within_lookback(&self, now: DateTime<Utc>) -> Vec<&SparkEvent> {
        if !self.config.enabled {
            return Vec::new();
        }
        let window = chrono::Duration::minutes(self.config.lookback_minutes as i64);
        self.recent_sparks
            .iter()
            .filter(|e| {
                DateTime::parse_from_rfc3339(&e.timestamp)
                    .map(|ts| now.signed_duration_since(ts.with_timezone(&Utc)) <= window)
                    .unwrap_or(true)
            })
            .collect()
    }

    /// JSON payload for distill_complete enrichment.
    pub fn lineage_payload(&self, session_id: &str, now: DateTime<Utc>) -> serde_json::Value {
        if !self.config.inject_spark_lineage {
            return serde_json::json!({});
        }
        let session_sparks = self.correlate_sparks(session_id);
        let recent = self.sparks_within_lookback(now);
        let mut anchor_ids: Vec<String> = recent
            .iter()
            .filter_map(|e| e.data.get("anchor_id").and_then(|v| v.as_str()))
            .map(str::to_string)
            .collect();
        anchor_ids.sort();
        anchor_ids.dedup();
        serde_json::json!({
            "session_spark_count": session_sparks.len(),
            "lookback_spark_count": recent.len(),
            "anchor_ids": anchor_ids,
            "lookback_minutes": self.config.lookback_minutes,
        })
    }

    pub fn ingest_synapse_events(&mut self, events: &[crate::synapse::SynapseEvent]) {
        if !self.config.enabled {
            return;
        }
        use crate::synapse::EventType;
        for event in events {
            if event.event_type != EventType::SparkComplete {
                continue;
            }
            let spark = SparkEvent {
                event_type: "spark_complete".to_string(),
                id: event.id.to_string(),
                source: format!("{:?}", event.source),
                timestamp: event.timestamp.to_rfc3339(),
                data: event.data.clone().unwrap_or(serde_json::json!({})),
            };
            self.register_spark(spark);
        }
    }

    pub fn persist_cache(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let cache = SparkDistillCache {
            updated_at: Some(Utc::now().to_rfc3339()),
            sparks: self.recent_sparks.clone(),
        };
        std::fs::write(path, serde_json::to_string_pretty(&cache)?)
    }

    pub fn load_cache(path: &Path) -> SparkDistillCache {
        if !path.is_file() {
            return SparkDistillCache::default();
        }
        std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    /// Load lineage for distill_complete from on-disk cache (CLI distill path).
    pub fn lineage_from_cache_file(cache_path: &Path, lookback_minutes: u64) -> serde_json::Value {
        let cache = Self::load_cache(cache_path);
        let now = Utc::now();
        let window = chrono::Duration::minutes(lookback_minutes as i64);
        let recent: Vec<&SparkEvent> = cache
            .sparks
            .iter()
            .filter(|e| {
                DateTime::parse_from_rfc3339(&e.timestamp)
                    .map(|ts| now.signed_duration_since(ts.with_timezone(&Utc)) <= window)
                    .unwrap_or(true)
            })
            .collect();
        let mut anchor_ids: Vec<String> = recent
            .iter()
            .filter_map(|e| e.data.get("anchor_id").and_then(|v| v.as_str()))
            .map(str::to_string)
            .collect();
        anchor_ids.sort();
        anchor_ids.dedup();
        serde_json::json!({
            "lookback_spark_count": recent.len(),
            "anchor_ids": anchor_ids,
            "cache_updated_at": cache.updated_at,
            "lookback_minutes": lookback_minutes,
        })
    }

    pub fn log_distill_correlations(&self, events: &[crate::synapse::SynapseEvent]) {
        use crate::synapse::EventType;
        let now = Utc::now();
        for event in events {
            if event.event_type != EventType::DistillComplete {
                continue;
            }
            let session_id = event
                .data
                .as_ref()
                .and_then(|d| d.get("session_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if session_id.is_empty() {
                continue;
            }
            let lineage = self.lineage_payload(session_id, now);
            let lookback = lineage
                .get("lookback_spark_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if lookback == 0 {
                tracing::debug!(session_id, "distill without spark lineage (lookback)");
            } else {
                tracing::info!(
                    session_id,
                    lookback_spark_count = lookback,
                    anchor_ids = ?lineage.get("anchor_ids"),
                    "distill correlated with spark lineage"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_register_and_correlate() {
        let config = SparkDistillBridgeConfig::default();
        let mut bridge = SparkDistillBridge::new(config);

        let spark_event = SparkEvent {
            event_type: "spark_complete".to_string(),
            id: "test-spark-001".to_string(),
            source: "test".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: serde_json::json!({
                "session_id": "test-session-001",
                "anchor_id": "anchor-a",
                "entities": ["entity-a", "entity-b"]
            }),
        };

        bridge.register_spark(spark_event);
        assert!(bridge.has_spark_lineage("test-session-001"));
        assert_eq!(bridge.spark_count(), 1);
        let payload = bridge.lineage_payload("test-session-001", Utc::now());
        assert_eq!(
            payload.get("anchor_ids").and_then(|v| v.as_array()).map(|a| a.len()),
            Some(1)
        );
    }

    #[test]
    fn test_bridge_disabled() {
        let config = SparkDistillBridgeConfig {
            enabled: false,
            ..SparkDistillBridgeConfig::default()
        };
        let mut bridge = SparkDistillBridge::new(config);

        let spark_event = SparkEvent {
            event_type: "spark_complete".to_string(),
            id: "test-spark-001".to_string(),
            source: "test".to_string(),
            timestamp: "2026-06-16T17:00:00Z".to_string(),
            data: serde_json::json!({ "session_id": "test-session-001" }),
        };

        bridge.register_spark(spark_event);
        assert_eq!(bridge.spark_count(), 0);
    }

    #[test]
    fn test_cache_roundtrip() {
        let dir = std::env::temp_dir().join(format!("spark_bridge_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cache.json");
        let mut bridge = SparkDistillBridge::new(SparkDistillBridgeConfig::default());
        bridge.register_spark(SparkEvent {
            event_type: "spark_complete".into(),
            id: "1".into(),
            source: "gzmo".into(),
            timestamp: Utc::now().to_rfc3339(),
            data: serde_json::json!({"anchor_id": "fact-123"}),
        });
        bridge.persist_cache(&path).unwrap();
        let lineage = SparkDistillBridge::lineage_from_cache_file(&path, 30);
        assert_eq!(
            lineage.get("lookback_spark_count").and_then(|v| v.as_u64()),
            Some(1)
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
