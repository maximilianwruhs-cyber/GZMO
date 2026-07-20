//! Night Lymph — compact overnight operator brief (filtrate, not the raw stream).
//!
//! Full spark/dream journals stay append-only elsewhere; this is what `gzmo status`
//! / Observatory should surface.

use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
const SCHEMA: &str = "gzmo.night_lymph/v1";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LymphDream {
    pub entities: usize,
    pub relations: usize,
    pub truths_promoted: usize,
    pub kg_entities: usize,
    pub kg_relations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LymphSpark {
    pub date: String,
    pub promoted: bool,
    pub kg_relations: usize,
    pub anchor_id: Option<String>,
    pub anchor_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightLymph {
    pub schema: String,
    pub night_id: String,
    pub updated_at: String,
    #[serde(default)]
    pub dream: Option<LymphDream>,
    #[serde(default)]
    pub sparks: Vec<LymphSpark>,
    #[serde(default)]
    pub immune_plan: Option<String>,
    #[serde(default)]
    pub immune_candidates: usize,
    #[serde(default)]
    pub note: String,
}

fn lymph_dir(vault_db: &Path) -> PathBuf {
    vault_db
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("night-lymph")
}

fn load_or_new(dir: &Path, night_id: &str) -> NightLymph {
    let path = dir.join("latest.json");
    if let Ok(raw) = std::fs::read_to_string(&path) {
        if let Ok(mut existing) = serde_json::from_str::<NightLymph>(&raw) {
            if existing.night_id == night_id {
                return existing;
            }
            // New night — keep sparks empty
            existing = NightLymph {
                schema: SCHEMA.into(),
                night_id: night_id.to_string(),
                updated_at: Utc::now().to_rfc3339(),
                dream: None,
                sparks: Vec::new(),
                immune_plan: None,
                immune_candidates: 0,
                note: String::new(),
            };
            return existing;
        }
    }
    NightLymph {
        schema: SCHEMA.into(),
        night_id: night_id.to_string(),
        updated_at: Utc::now().to_rfc3339(),
        dream: None,
        sparks: Vec::new(),
        immune_plan: None,
        immune_candidates: 0,
        note: "Compact overnight filtrate — see DREAMS.md for full spark stream.".into(),
    }
}

fn write_lymph(dir: &Path, lymph: &NightLymph) -> Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let text = serde_json::to_string_pretty(lymph)? + "\n";
    let dated = dir.join(format!("lymph-{}.json", lymph.night_id));
    std::fs::write(&dated, &text)?;
    std::fs::write(dir.join("latest.json"), &text)?;
    Ok(dated)
}

/// Merge dream stats + optional immune plan path into tonight's lymph.
pub fn record_dream(
    vault_db: &Path,
    night: NaiveDate,
    dream: LymphDream,
    immune_plan: Option<&Path>,
    immune_candidates: usize,
) -> Result<PathBuf> {
    let dir = lymph_dir(vault_db);
    let night_id = night.to_string();
    let mut lymph = load_or_new(&dir, &night_id);
    lymph.schema = SCHEMA.into();
    lymph.night_id = night_id;
    lymph.updated_at = Utc::now().to_rfc3339();
    lymph.dream = Some(dream);
    lymph.immune_candidates = immune_candidates;
    if let Some(p) = immune_plan {
        lymph.immune_plan = Some(p.display().to_string());
    }
    if lymph.note.is_empty() {
        lymph.note = "Compact overnight filtrate — see DREAMS.md for full spark stream.".into();
    }
    let path = write_lymph(&dir, &lymph)?;
    tracing::info!(path = %path.display(), "Night lymph updated (dream)");
    Ok(path)
}

/// Append one spark cycle summary (keeps last 8).
pub fn record_spark(vault_db: &Path, night: NaiveDate, spark: LymphSpark) -> Result<PathBuf> {
    let dir = lymph_dir(vault_db);
    let night_id = night.to_string();
    let mut lymph = load_or_new(&dir, &night_id);
    lymph.schema = SCHEMA.into();
    lymph.night_id = night_id;
    lymph.updated_at = Utc::now().to_rfc3339();
    lymph.sparks.push(spark);
    if lymph.sparks.len() > 8 {
        let skip = lymph.sparks.len() - 8;
        lymph.sparks = lymph.sparks.split_off(skip);
    }
    let path = write_lymph(&dir, &lymph)?;
    tracing::info!(path = %path.display(), "Night lymph updated (spark)");
    Ok(path)
}

/// Markdown one-pager for operator paste / status.
pub fn format_brief(lymph: &NightLymph) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Night lymph — {}\n\n", lymph.night_id));
    if let Some(d) = &lymph.dream {
        out.push_str(&format!(
            "- **Dream:** entities={} relations={} truths={} kg={}/{}\n",
            d.entities, d.relations, d.truths_promoted, d.kg_entities, d.kg_relations
        ));
    } else {
        out.push_str("- **Dream:** (not yet)\n");
    }
    out.push_str(&format!(
        "- **Sparks this night:** {}\n",
        lymph.sparks.len()
    ));
    for s in lymph.sparks.iter().rev().take(3) {
        let preview = s
            .anchor_preview
            .as_deref()
            .unwrap_or("(no preview)")
            .chars()
            .take(80)
            .collect::<String>();
        out.push_str(&format!(
            "  - promoted={} kg={} anchor=`{}`\n",
            s.promoted, s.kg_relations, preview
        ));
    }
    out.push_str(&format!(
        "- **Immune plan:** {} ({} candidates)\n",
        lymph.immune_plan.as_deref().unwrap_or("(none)"),
        lymph.immune_candidates
    ));
    out.push('\n');
    out.push_str(&lymph.note);
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dream_then_spark_merge_same_night() {
        let dir = std::env::temp_dir().join(format!("gzmo-lymph-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let vault = dir.join("vault.db");
        let night = NaiveDate::from_ymd_opt(2026, 7, 20).unwrap();
        record_dream(
            &vault,
            night,
            LymphDream {
                entities: 9,
                relations: 2,
                truths_promoted: 22,
                kg_entities: 9,
                kg_relations: 2,
            },
            None,
            3,
        )
        .unwrap();
        record_spark(
            &vault,
            night,
            LymphSpark {
                date: night.to_string(),
                promoted: true,
                kg_relations: 1,
                anchor_id: Some("abc".into()),
                anchor_preview: Some("[SYSTEM:GZMO] four layers".into()),
            },
        )
        .unwrap();
        let raw = std::fs::read_to_string(lymph_dir(&vault).join("latest.json")).unwrap();
        let lymph: NightLymph = serde_json::from_str(&raw).unwrap();
        assert_eq!(lymph.dream.unwrap().truths_promoted, 22);
        assert_eq!(lymph.sparks.len(), 1);
        assert_eq!(lymph.immune_candidates, 3);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
