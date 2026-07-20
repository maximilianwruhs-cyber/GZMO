//! Refractory Field — anti-monoculture pressure for spark selection.
//!
//! After a spark fires, the anchor (and its tag community) enter a decaying
//! refractory neighborhood. Soft-pick from top-K replaces greedy argmax so the
//! mid-band can breathe.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::SemanticFact;

const SCHEMA: &str = "gzmo.spark.refractory/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefractoryEntry {
    pub id: String,
    pub tags: Vec<String>,
    pub systemish: bool,
    pub selected_at: String,
    #[serde(default)]
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefractoryField {
    pub schema: String,
    pub entries: Vec<RefractoryEntry>,
}

impl Default for RefractoryField {
    fn default() -> Self {
        Self {
            schema: SCHEMA.into(),
            entries: Vec::new(),
        }
    }
}

pub fn spark_dir(vault_db: &Path) -> PathBuf {
    vault_db
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("spark")
}

pub fn refractory_path(vault_db: &Path) -> PathBuf {
    spark_dir(vault_db).join("refractory.json")
}

pub fn load_field(vault_db: &Path) -> RefractoryField {
    let path = refractory_path(vault_db);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_field(vault_db: &Path, field: &RefractoryField) -> std::io::Result<()> {
    let dir = spark_dir(vault_db);
    std::fs::create_dir_all(&dir)?;
    let text = serde_json::to_string_pretty(field)? + "\n";
    std::fs::write(refractory_path(vault_db), text)
}

pub fn extract_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find('[') {
        let after = &rest[start + 1..];
        if let Some(end) = after.find(']') {
            let tag = after[..end].trim().to_string();
            if !tag.is_empty() {
                tags.push(tag);
            }
            rest = &after[end + 1..];
        } else {
            break;
        }
    }
    tags
}

pub fn is_systemish(content: &str) -> bool {
    let upper = content.to_uppercase();
    upper.contains("[SYSTEM:")
        || upper.contains("SPARKENGINE")
        || upper.contains("DREAMENGINE")
        || upper.contains("SESSIONDISTILL")
}

fn hours_since(iso: &str) -> f64 {
    DateTime::parse_from_rfc3339(iso)
        .ok()
        .map(|t| {
            let t = t.with_timezone(&Utc);
            (Utc::now() - t).num_seconds().max(0) as f64 / 3600.0
        })
        .unwrap_or(1.0e9)
}

/// Multiplier in (0, 1]: lower = more suppressed by recent spark history.
pub fn refractory_multiplier(
    fact: &SemanticFact,
    field: &RefractoryField,
    half_life_hours: f64,
    strength: f64,
) -> f64 {
    if field.entries.is_empty() || half_life_hours <= 0.0 {
        return 1.0;
    }
    let tags = extract_tags(&fact.content);
    let systemish = is_systemish(&fact.content);
    let id = fact.id.to_string();
    let mut pen = 0.0_f64;

    let mut system_weight = 0.0;
    let mut total_weight = 0.0;

    for entry in &field.entries {
        let age_h = hours_since(&entry.selected_at);
        let decay = (-age_h / half_life_hours).exp();
        total_weight += decay;
        if entry.systemish {
            system_weight += decay;
        }
        if entry.id == id {
            pen = pen.max(decay);
            continue;
        }
        let overlap = entry.tags.iter().any(|t| tags.iter().any(|u| u == t));
        if overlap {
            pen = pen.max(decay * 0.65);
        }
    }

    let system_share = if total_weight > 0.0 {
        system_weight / total_weight
    } else {
        0.0
    };
    if systemish && system_share > 0.35 {
        pen = pen.max(system_share * 0.9);
    }

    let strength = strength.clamp(0.0, 1.0);
    (1.0 - pen * strength).clamp(0.05, 1.0)
}

/// Softmax sample among top-K scored candidates. `roll` in [0, 1).
pub fn soft_pick<T>(
    mut scored: Vec<(T, f64)>,
    top_k: usize,
    temperature: f64,
    roll: f64,
) -> Option<(T, f64)> {
    if scored.is_empty() {
        return None;
    }
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let k = top_k.max(1).min(scored.len());
    scored.truncate(k);
    if temperature <= 1e-9 || scored.len() == 1 {
        return scored.into_iter().next();
    }
    let max_s = scored[0].1;
    let weights: Vec<f64> = scored
        .iter()
        .map(|(_, s)| ((s - max_s) / temperature).exp())
        .collect();
    let sum: f64 = weights.iter().sum::<f64>().max(1e-12);
    let mut acc = 0.0;
    let target = roll.clamp(0.0, 0.999999) * sum;
    for (i, w) in weights.iter().enumerate() {
        acc += *w;
        if target <= acc {
            return Some(scored.swap_remove(i));
        }
    }
    scored.pop()
}

/// Deterministic roll from date + anchor id salt (reproducible overnight).
pub fn selection_roll(date: &str, salt: u64) -> f64 {
    let mut h = salt;
    for b in date.as_bytes() {
        h = h.wrapping_mul(16777619).wrapping_add(*b as u64);
    }
    ((h >> 11) as f64) / ((1u64 << 53) as f64)
}

pub fn record_selection(vault_db: &Path, fact: &SemanticFact, max_slots: usize) {
    let mut field = load_field(vault_db);
    field.schema = SCHEMA.into();
    field.entries.insert(
        0,
        RefractoryEntry {
            id: fact.id.to_string(),
            tags: extract_tags(&fact.content),
            systemish: is_systemish(&fact.content),
            selected_at: Utc::now().to_rfc3339(),
            preview: fact.content.chars().take(120).collect(),
        },
    );
    if field.entries.len() > max_slots {
        field.entries.truncate(max_slots);
    }
    let _ = save_field(vault_db, &field);
}

pub fn write_last_spark_report(
    vault_db: &Path,
    date: &str,
    promoted: bool,
    kg: usize,
    anchor_id: Option<Uuid>,
    anchor_preview: Option<&str>,
    score: Option<f64>,
) {
    let dir = spark_dir(vault_db);
    let _ = std::fs::create_dir_all(&dir);
    let payload = serde_json::json!({
        "schema": "gzmo.spark.report/v1",
        "date": date,
        "promoted": promoted,
        "kg_relations_written": kg,
        "anchor_id": anchor_id.map(|id| id.to_string()),
        "anchor_preview": anchor_preview,
        "selection_score": score,
        "updated_at": Utc::now().to_rfc3339(),
    });
    if let Ok(text) = serde_json::to_string_pretty(&payload) {
        let _ = std::fs::write(dir.join("last-spark-report.json"), text + "\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_pick_respects_top_score_at_zero_temp() {
        let picked = soft_pick(vec![("a", 1.0), ("b", 9.0), ("c", 2.0)], 3, 0.0, 0.5);
        assert_eq!(picked.unwrap().0, "b");
    }

    #[test]
    fn refractory_penalizes_same_id() {
        let id = Uuid::new_v4();
        let fact = SemanticFact {
            id,
            content: "[SYSTEM:GZMO] four layers".into(),
            embedding: vec![],
            half_life_days: 60.0,
            confidence: 1.0,
            confirmation_count: 1,
            decay_class: "CuratedVault".into(),
            created_at: Utc::now(),
            last_accessed_at: Utc::now(),
        };
        let field = RefractoryField {
            schema: SCHEMA.into(),
            entries: vec![RefractoryEntry {
                id: id.to_string(),
                tags: extract_tags(&fact.content),
                systemish: true,
                selected_at: Utc::now().to_rfc3339(),
                preview: String::new(),
            }],
        };
        let m = refractory_multiplier(&fact, &field, 72.0, 0.9);
        assert!(m < 0.3, "same-id should be heavily suppressed, got {m}");
    }

    #[test]
    fn system_homophily_suppresses_systemish_when_field_saturated() {
        let fact = SemanticFact {
            id: Uuid::new_v4(),
            content: "[SYSTEM:SparkEngine] meta".into(),
            embedding: vec![],
            half_life_days: 60.0,
            confidence: 1.0,
            confirmation_count: 1,
            decay_class: "CuratedVault".into(),
            created_at: Utc::now(),
            last_accessed_at: Utc::now(),
        };
        let mut entries = Vec::new();
        for _ in 0..6 {
            entries.push(RefractoryEntry {
                id: Uuid::new_v4().to_string(),
                tags: vec!["SYSTEM:GZMO".into()],
                systemish: true,
                selected_at: Utc::now().to_rfc3339(),
                preview: String::new(),
            });
        }
        let field = RefractoryField {
            schema: SCHEMA.into(),
            entries,
        };
        let m = refractory_multiplier(&fact, &field, 72.0, 0.85);
        assert!(m < 0.5, "system saturation should suppress, got {m}");
    }
}
