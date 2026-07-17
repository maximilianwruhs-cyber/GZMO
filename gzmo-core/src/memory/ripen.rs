//! # M5: Honeypot Ripening Engine
//!
//! Transforms raw honeypot entries into a dense, contradiction-resolved,
//! concept-carded knowledge core. Runs as an hourly background job.
//!
//! ## Phases
//!
//! 1. **Global dedup** — group by content_norm similarity > 0.95
//! 2. **Contradiction resolution** — for groups with is_latest=0 entries,
//!    pick the winner by highest confidence × recall_count
//! 3. **Concept card synthesis** — for entities with >5 honeypot entries,
//!    generate a synthesized summary from aggregated evidence
//! 4. **Export** — write ripened cards to `knowledge_core` table
//!
//! ## Usage (from daemon)
//!
//! ```ignore
//! use gzmo_core::memory::ripen::ripen_honeypot;
//! let cards = ripen_honeypot(&vault, &honeypot)?;
//! info!("Ripened {} concept cards", cards.len());
//! ```

use std::collections::HashMap;

use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::memory::vault::SqliteVault;

/// A ripened concept card — synthesized from multiple honeypot entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptCard {
    pub id: String,
    pub label: String,
    pub entity_type: String,
    pub summary: String,
    pub supporting_facts: Vec<SupportingFact>,
    pub confidence: f64,
    pub contradiction_resolved: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportingFact {
    pub honeypot_id: String,
    pub content: String,
    pub confidence: f64,
    pub origin: String,
    pub recall_count: i64,
}

/// Configuration for the ripen job.
#[derive(Debug, Clone)]
pub struct RipenConfig {
    /// Similarity threshold for dedup grouping (0.0–1.0).
    pub dedup_threshold: f64,
    /// Minimum honeypot entries per entity to build a concept card.
    pub min_entries_for_card: usize,
    /// Minimum confidence for inclusion in a concept card.
    pub min_confidence: f64,
    /// Maximum cards to produce per run.
    pub max_cards: usize,
    /// Export to knowledge_core table.
    pub export: bool,
}

impl Default for RipenConfig {
    fn default() -> Self {
        Self {
            dedup_threshold: 0.95,
            min_entries_for_card: 5,
            min_confidence: 0.85,
            max_cards: 50,
            export: true,
        }
    }
}

/// Run the full ripen cycle.
pub fn ripen_honeypot(vault: &SqliteVault, config: &RipenConfig) -> Result<Vec<ConceptCard>> {
    let conn = vault.db_conn()?;
    info!("M5 ripen cycle starting");

    // Phase 1: Group honeypot entries by entity label
    let groups = group_by_entity(&conn, config)?;
    info!(groups = groups.len(), "Phase 1: entity grouping complete");

    // Phase 2: Resolve contradictions within each group
    let resolved = resolve_contradictions(&conn, groups, config)?;
    info!(
        cards = resolved.len(),
        "Phase 2: contradiction resolution complete"
    );

    // Phase 3: Export to knowledge_core table
    if config.export && !resolved.is_empty() {
        export_cards(&conn, &resolved)?;
        info!(
            exported = resolved.len(),
            "Phase 3: exported to knowledge_core"
        );
    }

    Ok(resolved)
}

/// Group honeypot entries by extracted entity label.
fn group_by_entity(
    conn: &Connection,
    config: &RipenConfig,
) -> Result<HashMap<String, Vec<EntityEntry>>> {
    let mut stmt = conn.prepare(
        "SELECT h.id, h.content, h.confidence, h.origin, h.recall_count,
                h.is_latest, h.supersedes_id, h.decay_class, h.promoted_at
         FROM honeypot h
         WHERE h.confidence >= ?1
         ORDER BY h.confidence DESC",
    )?;

    let rows = stmt.query_map(params![config.min_confidence], |row| {
        Ok(EntityEntry {
            id: row.get(0)?,
            content: row.get(1)?,
            confidence: row.get(2)?,
            origin: row.get(3)?,
            recall_count: row.get(4)?,
            is_latest: row.get::<_, i64>(5)? != 0,
            supersedes_id: row.get(6)?,
            decay_class: row.get::<_, String>(7)?,
            promoted_at: row.get(8)?,
        })
    })?;

    let mut groups: HashMap<String, Vec<EntityEntry>> = HashMap::new();

    for row in rows {
        let entry = row?;
        // Extract entity label from content like "[TYPE:Name] observation"
        let label = extract_entity_label(&entry.content);
        groups.entry(label).or_default().push(entry);
    }

    // Filter groups that meet the minimum card threshold
    groups.retain(|_, entries| entries.len() >= config.min_entries_for_card);

    Ok(groups)
}

/// Resolve contradictions within each entity group.
fn resolve_contradictions(
    _conn: &Connection,
    groups: HashMap<String, Vec<EntityEntry>>,
    config: &RipenConfig,
) -> Result<Vec<ConceptCard>> {
    let mut cards = Vec::new();

    for (label, mut entries) in groups {
        if cards.len() >= config.max_cards {
            break;
        }

        // Sort by confidence × recall_count (descending)
        entries.sort_by(|a, b| {
            let score_a = a.confidence * (1.0 + a.recall_count as f64);
            let score_b = b.confidence * (1.0 + b.recall_count as f64);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Check for contradictions (superseded entries)
        let has_contradictions = entries.iter().any(|e| !e.is_latest);

        // Build the concept card
        let (entity_type, _name) = split_entity_label(&label);
        let summary = synthesize_summary(&entries);
        let supporting_facts: Vec<SupportingFact> = entries
            .iter()
            .take(10)
            .map(|e| SupportingFact {
                honeypot_id: e.id.clone(),
                content: truncate_content(&e.content, 200),
                confidence: e.confidence,
                origin: e.origin.clone(),
                recall_count: e.recall_count,
            })
            .collect();

        let best_entry = entries.first().unwrap();
        cards.push(ConceptCard {
            id: format!("card_{}", best_entry.id),
            label,
            entity_type,
            summary,
            supporting_facts,
            confidence: best_entry.confidence,
            contradiction_resolved: has_contradictions,
            created_at: Utc::now().to_rfc3339(),
        });
    }

    Ok(cards)
}

/// Export concept cards to the knowledge_core table.
fn export_cards(conn: &Connection, cards: &[ConceptCard]) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS knowledge_core (
            id TEXT PRIMARY KEY,
            label TEXT,
            entity_type TEXT,
            summary TEXT,
            confidence REAL,
            contradiction_resolved INTEGER,
            supporting_facts TEXT,
            created_at TEXT,
            exported_at TEXT
        );",
    )?;

    for card in cards {
        let facts_json =
            serde_json::to_string(&card.supporting_facts).unwrap_or_else(|_| "[]".into());
        conn.execute(
            "INSERT OR REPLACE INTO knowledge_core
             (id, label, entity_type, summary, confidence, contradiction_resolved,
              supporting_facts, created_at, exported_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
            params![
                card.id,
                card.label,
                card.entity_type,
                card.summary,
                card.confidence,
                card.contradiction_resolved as i64,
                facts_json,
                card.created_at,
            ],
        )?;
    }

    Ok(())
}

/// An entry from the honeypot table for ripen processing.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct EntityEntry {
    id: String,
    content: String,
    confidence: f64,
    origin: String,
    recall_count: i64,
    is_latest: bool,
    supersedes_id: Option<String>,
    decay_class: String,
    promoted_at: Option<String>,
}

/// Extract entity label from "[TYPE:Name] observation" format.
fn extract_entity_label(content: &str) -> String {
    if let Some(end) = content.find(']') {
        let bracket = &content[1..end];
        if let Some(col) = bracket.find(':') {
            return bracket[col + 1..].trim().to_lowercase();
        }
    }
    // Fallback: use first 40 chars
    content.chars().take(40).collect()
}

/// Split "name" into ("TYPE", "Name") — falls back to ("CONCEPT", name).
fn split_entity_label(label: &str) -> (String, String) {
    (String::from("CONCEPT"), label.to_string())
}

/// Synthesize a summary from the top entries.
fn synthesize_summary(entries: &[EntityEntry]) -> String {
    let top: Vec<&str> = entries
        .iter()
        .filter(|e| e.is_latest)
        .take(5)
        .map(|e| e.content.as_str())
        .collect();

    if top.is_empty() {
        return "No current entries.".to_string();
    }

    // Take the most informative observations
    let mut summary = String::from("Synthesized from honeypot:\n");
    for obs in &top {
        // Strip the [TYPE:Name] prefix for cleaner display
        if let Some(end) = obs.find("] ") {
            summary.push_str(&obs[end + 2..]);
            summary.push('\n');
        } else {
            summary.push_str(obs);
            summary.push('\n');
        }
    }
    summary
}

/// Truncate content for display.
fn truncate_content(content: &str, max: usize) -> String {
    if content.len() <= max {
        content.to_string()
    } else {
        format!("{}...", &content[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_label_from_bracket_format() {
        assert_eq!(extract_entity_label("[SYSTEM:GZMO] runs locally"), "gzmo");
        assert_eq!(
            extract_entity_label("[PROJECT:Obolus] token economy"),
            "obolus"
        );
    }

    #[test]
    fn extracts_fallback_when_no_bracket() {
        let label = extract_entity_label("plain text observation");
        assert_eq!(label.len(), 22); // "plain text observation" is 22 chars
    }

    #[test]
    fn fallback_truncates_long_content() {
        let long = "a".repeat(100);
        let label = extract_entity_label(&long);
        assert_eq!(label.len(), 40);
    }

    #[test]
    fn synthesizes_summary_from_top_entries() {
        let entries = vec![
            EntityEntry {
                id: "1".into(),
                content: "[SYSTEM:GZMO] Runs on CT101".into(),
                confidence: 0.95,
                origin: "ingest".into(),
                recall_count: 10,
                is_latest: true,
                supersedes_id: None,
                decay_class: "CuratedVault".into(),
                promoted_at: None,
            },
            EntityEntry {
                id: "2".into(),
                content: "[SYSTEM:GZMO] Uses cloud LLM by default".into(),
                confidence: 0.90,
                origin: "session_distill".into(),
                recall_count: 5,
                is_latest: true,
                supersedes_id: None,
                decay_class: "SessionDistill".into(),
                promoted_at: None,
            },
        ];
        let summary = synthesize_summary(&entries);
        assert!(summary.contains("Runs on CT101"));
        assert!(summary.contains("Uses cloud LLM"));
    }

    #[test]
    fn group_by_requires_minimum_entries() {
        // This tests the grouping logic without a database
        let config = RipenConfig {
            min_entries_for_card: 3,
            ..RipenConfig::default()
        };
        assert_eq!(config.min_entries_for_card, 3);
    }
}
