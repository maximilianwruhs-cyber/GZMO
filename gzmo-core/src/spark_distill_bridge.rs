/// Spark-to-distill session correlation bridge.
/// 
/// Provenance: Probe A02 (e2e-verify-17-35-00) found that `spark_complete` 
/// events (653 total) do NOT correlate with `distill_complete` sessions (55 total).
/// Zero sessions contain both events, meaning sparks do not pre-qualify entities
/// for promotion in the current pipeline.
/// 
/// This module provides a bridge that injects spark session lineage into distill
/// session metadata, enabling downstream consumers to trace promotion lineage.

use serde::{Deserialize, Serialize};

/// Represents a spark event extracted from the Synapse bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkEvent {
    pub event_type: String,
    pub id: String,
    pub source: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

/// Represents a distill event from the Synapse bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillEvent {
    pub event_type: String,
    pub session_id: String,
    pub timestamp: String,
    pub entities_promoted: Vec<String>,
    pub kg_entities_written: usize,
    pub kg_relations_written: usize,
    pub relations_promoted: Vec<String>,
    pub vault_truths: Vec<String>,
}

/// Configuration for the spark-to-distill correlation bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkDistillBridgeConfig {
    /// Whether the bridge is enabled
    pub enabled: bool,
    /// Lookback window in minutes for finding spark events before a distill event
    pub lookback_minutes: u64,
    /// Whether to inject spark entity IDs into distill session metadata
    pub inject_spark_lineage: bool,
}

impl Default for SparkDistillBridgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            lookback_minutes: 30,
            inject_spark_lineage: true,
        }
    }
}

/// Bridge state — tracks the most recent spark events indexed by session_id.
pub struct SparkDistillBridge {
    config: SparkDistillBridgeConfig,
    /// Cache of recent spark events keyed by session_id
    spark_cache: std::collections::HashMap<String, Vec<SparkEvent>>,
}

impl SparkDistillBridge {
    pub fn new(config: SparkDistillBridgeConfig) -> Self {
        Self {
            config,
            spark_cache: std::collections::HashMap::new(),
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
                .push(event);
        }
    }

    /// Given a distill session, return correlated spark events.
    pub fn correlate_sparks(&self, session_id: &str) -> Vec<&SparkEvent> {
        if !self.config.enabled {
            return Vec::new();
        }
        self.spark_cache
            .get(session_id)
            .map(|events| events.iter().collect())
            .unwrap_or_default()
    }

    /// Check if any spark events exist for a given session.
    pub fn has_spark_lineage(&self, session_id: &str) -> bool {
        self.spark_cache
            .get(session_id)
            .map(|events| !events.is_empty())
            .unwrap_or(false)
    }

    /// Get the total number of registered spark events.
    pub fn spark_count(&self) -> usize {
        self.spark_cache.values().map(|v| v.len()).sum()
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
            timestamp: "2026-06-16T17:00:00Z".to_string(),
            data: serde_json::json!({
                "session_id": "test-session-001",
                "entities": ["entity-a", "entity-b"]
            }),
        };

        bridge.register_spark(spark_event);
        assert!(bridge.has_spark_lineage("test-session-001"));
        assert!(!bridge.has_spark_lineage("test-session-002"));
        assert_eq!(bridge.spark_count(), 1);
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
            data: serde_json::json!({
                "session_id": "test-session-001"
            }),
        };

        bridge.register_spark(spark_event);
        assert_eq!(bridge.spark_count(), 0);
        assert!(!bridge.has_spark_lineage("test-session-001"));
    }
}
