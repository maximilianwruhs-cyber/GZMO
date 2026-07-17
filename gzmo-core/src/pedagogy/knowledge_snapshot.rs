//! Knowledge state snapshots for pedagogy oscillation bus events.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VaultKnowledgeMetrics {
    pub semantic_vault_count: u64,
    pub honeypot_count: u64,
    pub discovery_sourced_count: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeStateSnapshot {
    #[serde(default)]
    pub known_nodes: Vec<String>,
    #[serde(default)]
    pub open_gaps: Vec<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vault_metrics: Option<VaultKnowledgeMetrics>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeDelta {
    #[serde(default)]
    pub added: Vec<String>,
    #[serde(default)]
    pub changed: Vec<String>,
    #[serde(default)]
    pub removed: Vec<String>,
}

/// Empty baseline when no handoff is available.
pub fn empty_knowledge_state() -> KnowledgeStateSnapshot {
    KnowledgeStateSnapshot {
        known_nodes: Vec::new(),
        open_gaps: Vec::new(),
        source: "empty".to_string(),
        vault_metrics: None,
    }
}

/// Read vault/honeypot counts from SQLite (sync, for oscillation metrics).
pub fn vault_metrics_from_path(vault_db: &Path) -> VaultKnowledgeMetrics {
    let mut metrics = VaultKnowledgeMetrics::default();
    let Ok(conn) =
        rusqlite::Connection::open_with_flags(vault_db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
    else {
        return metrics;
    };

    metrics.semantic_vault_count = conn
        .query_row("SELECT COUNT(*) FROM semantic_vault", [], |r| r.get(0))
        .unwrap_or(0);
    metrics.honeypot_count = conn
        .query_row(
            "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    metrics.discovery_sourced_count = conn
        .query_row(
            "SELECT COUNT(*) FROM semantic_vault WHERE source_file LIKE 'sessions/discovery-%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    metrics
}

/// Build a knowledge snapshot from live vault counts.
pub fn knowledge_state_from_vault(vault_db: &Path) -> KnowledgeStateSnapshot {
    let m = vault_metrics_from_path(vault_db);
    KnowledgeStateSnapshot {
        known_nodes: vec![
            format!("vault:{}", m.semantic_vault_count),
            format!("honeypot:{}", m.honeypot_count),
            format!("discovery_sourced:{}", m.discovery_sourced_count),
        ],
        open_gaps: Vec::new(),
        source: "vault_metrics".to_string(),
        vault_metrics: Some(m),
    }
}

/// Prefer handoff JSON when present; otherwise vault metrics.
pub fn knowledge_state_for_cycle_start(vault_db: &Path) -> KnowledgeStateSnapshot {
    let handoff = knowledge_state_from_handoff_env();
    if !handoff.known_nodes.is_empty() || !handoff.open_gaps.is_empty() {
        return handoff;
    }
    knowledge_state_from_vault(vault_db)
}

/// Delta between two snapshots (handoff nodes + vault metric deltas).
pub fn compute_knowledge_delta(
    before: &KnowledgeStateSnapshot,
    after: &KnowledgeStateSnapshot,
) -> KnowledgeDelta {
    let mut delta = KnowledgeDelta::default();

    let before_set: std::collections::HashSet<_> = before.known_nodes.iter().collect();
    let after_set: std::collections::HashSet<_> = after.known_nodes.iter().collect();
    for node in &after.known_nodes {
        if !before_set.contains(node) {
            delta.added.push(node.clone());
        }
    }
    for node in &before.known_nodes {
        if !after_set.contains(node) {
            delta.removed.push(node.clone());
        }
    }

    if let (Some(b), Some(a)) = (&before.vault_metrics, &after.vault_metrics) {
        if a.semantic_vault_count > b.semantic_vault_count {
            delta.changed.push(format!(
                "semantic_vault:+{}",
                a.semantic_vault_count - b.semantic_vault_count
            ));
        }
        if a.honeypot_count > b.honeypot_count {
            delta
                .changed
                .push(format!("honeypot:+{}", a.honeypot_count - b.honeypot_count));
        }
        if a.discovery_sourced_count > b.discovery_sourced_count {
            delta.changed.push(format!(
                "discovery_sourced:+{}",
                a.discovery_sourced_count - b.discovery_sourced_count
            ));
        }
    }

    delta
}

/// Load v2 handoff JSON from `GZMO_HANDOFF_PATH` env or explicit path.
pub fn knowledge_state_from_handoff_path(path: &Path) -> KnowledgeStateSnapshot {
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return empty_knowledge_state(),
    };
    let v: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return empty_knowledge_state(),
    };
    knowledge_state_from_handoff_value(&v)
}

pub fn knowledge_state_from_handoff_env() -> KnowledgeStateSnapshot {
    let Some(path) = std::env::var("GZMO_HANDOFF_PATH")
        .ok()
        .filter(|s| !s.is_empty())
    else {
        return empty_knowledge_state();
    };
    knowledge_state_from_handoff_path(Path::new(&path))
}

pub fn knowledge_state_from_handoff_value(v: &Value) -> KnowledgeStateSnapshot {
    if let Some(ks) = v.get("knowledge_state") {
        if let Ok(mut snap) = serde_json::from_value::<KnowledgeStateSnapshot>(ks.clone()) {
            if snap.source.is_empty() {
                snap.source = "socratic_handoff".to_string();
            }
            return snap;
        }
    }

    let concepts = string_array(v.get("concepts_established"));
    let gaps = string_array(v.get("gaps_identified"));
    if concepts.is_empty() && gaps.is_empty() {
        return empty_knowledge_state();
    }

    KnowledgeStateSnapshot {
        known_nodes: concepts,
        open_gaps: gaps,
        source: "socratic_handoff".to_string(),
        vault_metrics: None,
    }
}

pub fn empty_knowledge_delta() -> KnowledgeDelta {
    KnowledgeDelta::default()
}

pub fn snapshot_to_json(s: &KnowledgeStateSnapshot) -> Value {
    serde_json::to_value(s).unwrap_or_else(|_| json!({}))
}

pub fn delta_to_json(d: &KnowledgeDelta) -> Value {
    serde_json::to_value(d).unwrap_or_else(|_| json!({"added":[],"changed":[],"removed":[]}))
}

fn string_array(v: Option<&Value>) -> Vec<String> {
    v.and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handoff_v2_maps_concepts_and_gaps() {
        let v = json!({
            "version": 2,
            "concepts_established": ["bounded chaos"],
            "gaps_identified": ["novel application metric"]
        });
        let snap = knowledge_state_from_handoff_value(&v);
        assert_eq!(snap.known_nodes, vec!["bounded chaos"]);
        assert_eq!(snap.open_gaps, vec!["novel application metric"]);
        assert_eq!(snap.source, "socratic_handoff");
    }

    #[test]
    fn compute_delta_tracks_vault_growth() {
        let before = KnowledgeStateSnapshot {
            known_nodes: vec!["vault:10".to_string()],
            open_gaps: vec![],
            source: "vault_metrics".to_string(),
            vault_metrics: Some(VaultKnowledgeMetrics {
                semantic_vault_count: 10,
                honeypot_count: 5,
                discovery_sourced_count: 0,
            }),
        };
        let after = KnowledgeStateSnapshot {
            known_nodes: vec!["vault:12".to_string()],
            open_gaps: vec![],
            source: "vault_metrics".to_string(),
            vault_metrics: Some(VaultKnowledgeMetrics {
                semantic_vault_count: 12,
                honeypot_count: 6,
                discovery_sourced_count: 1,
            }),
        };
        let delta = compute_knowledge_delta(&before, &after);
        assert!(delta
            .changed
            .iter()
            .any(|c| c.contains("semantic_vault:+2")));
        assert!(delta
            .changed
            .iter()
            .any(|c| c.contains("discovery_sourced:+1")));
    }
}
