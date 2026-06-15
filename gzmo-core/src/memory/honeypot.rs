//! Curated honeypot layer — subset of vault facts eligible for Qdrant / recall (M2).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Serialize;

use crate::types::ExtractedTruth;

/// Minimum confidence aligned with `[ingest] min_confidence` and vault promote gate.
/// Tuned down from 0.85 (2026-06-15) to reduce false negatives on derived facts.
pub const HONEYPOT_MIN_CONFIDENCE: f32 = 0.80;

pub const HONEYPOT_REJECT_LOG: &str = "data/honeypot_reject.jsonl";

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HoneypotRejectReason {
    LowConfidence { got: f32, min: f32 },
    MissingSourceFile,
    ExcludedSourcePattern { pattern: String },
    RelationRow,
    Boilerplate,
}

pub fn honeypot_eligibility(truth: &ExtractedTruth) -> Result<(), HoneypotRejectReason> {
    if truth.confidence < HONEYPOT_MIN_CONFIDENCE {
        return Err(HoneypotRejectReason::LowConfidence {
            got: truth.confidence,
            min: HONEYPOT_MIN_CONFIDENCE,
        });
    }
    let Some(sf) = &truth.source_file else {
        return Err(HoneypotRejectReason::MissingSourceFile);
    };
    if sf.trim().is_empty() {
        return Err(HoneypotRejectReason::MissingSourceFile);
    }
    let sf_lower = sf.to_lowercase();
    for pattern in ["chat_history", "chat_session", "quelltext", "sources"] {
        if sf_lower.contains(pattern) {
            return Err(HoneypotRejectReason::ExcludedSourcePattern {
                pattern: pattern.to_string(),
            });
        }
    }
    let lower = truth.content.to_lowercase();
    if lower.starts_with("[relation:") {
        return Err(HoneypotRejectReason::RelationRow);
    }
    if is_boilerplate(&lower) {
        return Err(HoneypotRejectReason::Boilerplate);
    }
    Ok(())
}

pub fn qualifies_for_honeypot(truth: &ExtractedTruth) -> bool {
    honeypot_eligibility(truth).is_ok()
}

pub fn append_reject_log(
    log_path: &Path,
    reason: &HoneypotRejectReason,
    truth: &ExtractedTruth,
    vault_id: &str,
) -> std::io::Result<()> {
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let preview: String = truth.content.chars().take(160).collect();
    let line = serde_json::json!({
        "ts": Utc::now().to_rfc3339(),
        "vault_id": vault_id,
        "reason": reason,
        "confidence": truth.confidence,
        "source_file": truth.source_file,
        "content_preview": preview,
    });
    let mut file = OpenOptions::new().create(true).append(true).open(log_path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

/// Queue a vault row for operator review after honeypot rejection.
pub fn enqueue_review(
    conn: &Connection,
    vault_id: &str,
    reason: &HoneypotRejectReason,
    truth: &ExtractedTruth,
) -> std::io::Result<()> {
    let preview: String = truth.content.chars().take(200).collect();
    let reason_json = serde_json::to_string(reason).unwrap_or_else(|_| "\"unknown\"".into());
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO honeypot_review_queue
            (vault_id, reason, content_preview, confidence, source_file, queued_at, reviewed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)
         ON CONFLICT(vault_id) DO UPDATE SET
            reason = excluded.reason,
            content_preview = excluded.content_preview,
            confidence = excluded.confidence,
            source_file = excluded.source_file,
            queued_at = excluded.queued_at,
            reviewed = 0",
        params![
            vault_id,
            reason_json,
            preview,
            truth.confidence as f64,
            truth.source_file,
            now,
        ],
    )
    .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(())
}

/// List pending review queue rows (newest first).
pub fn list_review_queue(conn: &Connection, limit: usize) -> rusqlite::Result<Vec<(String, String, String, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT vault_id, reason, content_preview, confidence
         FROM honeypot_review_queue
         WHERE reviewed = 0
         ORDER BY queued_at DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, f64>(3)?,
        ))
    })?;
    rows.collect()
}

/// Force-promote a queued vault row into honeypot (operator override).
pub fn promote_reviewed(
    conn: &Connection,
    vault_id: &str,
    embedding: &[u8],
    content_norm: &str,
    origin: &str,
) -> anyhow::Result<()> {
    let row = conn.query_row(
        "SELECT content, confidence, source_file, decay_class
         FROM semantic_vault WHERE id = ?1",
        params![vault_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    )?;
    let (content, confidence, source_file, _decay_class) = row;
    let truth = ExtractedTruth {
        id: uuid::Uuid::new_v4(),
        content,
        confidence: confidence as f32,
        mmr_score: 0.0,
        source_date: chrono::Utc::now().date_naive(),
        decay_class: crate::types::DecayClass::CuratedVault,
        source_file,
        evidence: None,
    };
    upsert_honeypot_row(conn, vault_id, &truth, embedding, content_norm, origin)?;
    conn.execute(
        "UPDATE honeypot_review_queue SET reviewed = 1 WHERE vault_id = ?1",
        params![vault_id],
    )?;
    Ok(())
}

fn is_boilerplate(lower: &str) -> bool {
    lower.contains("sources do not contain")
        || lower.contains("migration_id")
        || lower.contains("takeout drive")
}

fn container_tag_for_origin(origin: &str) -> &str {
    let lower = origin.to_ascii_lowercase();
    if lower.contains("wuerfel") || lower.contains("dice_cascade") || lower.contains("wuerfel-cron") {
        crate::bibliothek::WUERFEL_SANDBOX_TAG
    } else {
        "obolus"
    }
}

/// Insert a new honeypot row with lifecycle metadata (new vault id — no upsert overwrite).
pub fn insert_honeypot_lifecycle(
    conn: &Connection,
    vault_id: &str,
    truth: &ExtractedTruth,
    embedding: &[u8],
    content_norm: &str,
    origin: &str,
    graph_rel: Option<&str>,
    supersedes_id: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let decay = format!("{:?}", truth.decay_class);
    conn.execute(
        "INSERT INTO honeypot (
            id, vault_id, content, content_norm, embedding, origin, memory_type,
            graph_rel, supersedes_id, verify_pass, confidence, decay_class,
            source_file, container_tag, promoted_at, is_latest, recall_count
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, 'fact', ?7, ?8, 1, ?9, ?10, ?11, ?12, 1, 0
        )",
        params![
            vault_id,
            vault_id,
            truth.content,
            content_norm,
            embedding,
            origin,
            graph_rel,
            supersedes_id,
            truth.confidence as f64,
            decay,
            truth.source_file,
            container_tag_for_origin(origin),
            now,
        ],
    )?;
    sync_honeypot_fts_row(conn, vault_id, &truth.content, content_norm)?;
    crate::memory::profile::invalidate_profile_cache(Some("obolus"));
    Ok(())
}

/// Upsert one honeypot row inside an open vault transaction (`id` = `vault_id`).
pub fn upsert_honeypot_row(
    conn: &Connection,
    vault_id: &str,
    truth: &ExtractedTruth,
    embedding: &[u8],
    content_norm: &str,
    origin: &str,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let decay = format!("{:?}", truth.decay_class);
    conn.execute(
        "INSERT INTO honeypot (
            id, vault_id, content, content_norm, embedding, origin, memory_type,
            verify_pass, confidence, decay_class, source_file, container_tag,
            promoted_at, is_latest, recall_count
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, 'fact', 1, ?7, ?8, ?9, ?10, ?11, 1, 0
        )
        ON CONFLICT(id) DO UPDATE SET
            content = excluded.content,
            content_norm = excluded.content_norm,
            embedding = excluded.embedding,
            origin = excluded.origin,
            confidence = MAX(confidence, excluded.confidence),
            decay_class = excluded.decay_class,
            source_file = excluded.source_file,
            container_tag = excluded.container_tag,
            promoted_at = excluded.promoted_at,
            is_latest = 1",
        params![
            vault_id,
            vault_id,
            truth.content,
            content_norm,
            embedding,
            origin,
            truth.confidence as f64,
            decay,
            truth.source_file,
            container_tag_for_origin(origin),
            now,
        ],
    )?;
    sync_honeypot_fts_row(conn, vault_id, &truth.content, content_norm)?;
    crate::memory::profile::invalidate_profile_cache(Some("obolus"));
    Ok(())
}

/// Keep FTS5 index aligned with honeypot rows (triggers removed in schema v4).
pub fn sync_honeypot_fts_row(
    conn: &Connection,
    vault_id: &str,
    content: &str,
    content_norm: &str,
) -> Result<()> {
    let rowid: i64 = conn.query_row(
        "SELECT rowid FROM honeypot WHERE id = ?1",
        params![vault_id],
        |r| r.get(0),
    )?;
    conn.execute("DELETE FROM honeypot_fts WHERE rowid = ?1", params![rowid])?;
    conn.execute(
        "INSERT INTO honeypot_fts(rowid, content, content_norm) VALUES (?1, ?2, ?3)",
        params![rowid, content, content_norm],
    )?;
    Ok(())
}

/// Upsert one evidence row inside an open vault transaction.
pub fn upsert_evidence_row(
    conn: &Connection,
    evidence_id: &str,
    fact_id: &str,
    source_file: Option<&str>,
    evidence_text: &str,
    evidence_norm: &str,
    char_start: Option<usize>,
    char_end: Option<usize>,
    quote_verifier: Option<&str>,
    embedding: &[u8],
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let char_start_val = char_start.map(|x| x as i64);
    let char_end_val = char_end.map(|x| x as i64);

    conn.execute(
        "INSERT INTO evidence (
            id, fact_id, source_file, evidence_text, evidence_norm,
            char_start, char_end, quote_verifier, embedding,
            verify_pass, created_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10
        )
        ON CONFLICT(id) DO UPDATE SET
            fact_id = excluded.fact_id,
            source_file = excluded.source_file,
            evidence_text = excluded.evidence_text,
            evidence_norm = excluded.evidence_norm,
            char_start = excluded.char_start,
            char_end = excluded.char_end,
            quote_verifier = excluded.quote_verifier,
            embedding = excluded.embedding,
            created_at = excluded.created_at",
        params![
            evidence_id,
            fact_id,
            source_file,
            evidence_text,
            evidence_norm,
            char_start_val,
            char_end_val,
            quote_verifier,
            embedding,
            now,
        ],
    )?;
    sync_evidence_fts_row(conn, evidence_id, evidence_text, evidence_norm)?;
    Ok(())
}

/// Keep FTS5 index aligned with evidence rows.
pub fn sync_evidence_fts_row(
    conn: &Connection,
    evidence_id: &str,
    evidence_text: &str,
    evidence_norm: &str,
) -> Result<()> {
    let rowid: i64 = conn.query_row(
        "SELECT rowid FROM evidence WHERE id = ?1",
        params![evidence_id],
        |r| r.get(0),
    )?;
    conn.execute("DELETE FROM evidence_fts WHERE rowid = ?1", params![rowid])?;
    conn.execute(
        "INSERT INTO evidence_fts(rowid, evidence_text, evidence_norm) VALUES (?1, ?2, ?3)",
        params![rowid, evidence_text, evidence_norm],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DecayClass;
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn truth(content: &str, conf: f32, source: Option<&str>) -> ExtractedTruth {
        ExtractedTruth {
            id: Uuid::new_v4(),
            content: content.to_string(),
            confidence: conf,
            mmr_score: 0.0,
            source_date: NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
            decay_class: DecayClass::CuratedVault,
            source_file: source.map(str::to_string),
            evidence: None,
        }
    }

    #[test]
    fn qualifies_with_source_and_confidence() {
        assert!(qualifies_for_honeypot(&truth(
            "[SYSTEM:Backup] uses ZFS",
            0.9,
            Some("wave_01_foo.md"),
        )));
    }

    #[test]
    fn rejects_relation_and_boilerplate() {
        assert!(!qualifies_for_honeypot(&truth(
            "[RELATION:USES] A → B",
            0.9,
            Some("wave_01_foo.md"),
        )));
        assert!(!qualifies_for_honeypot(&truth(
            "sources do not contain X",
            0.9,
            Some("wave_01_foo.md"),
        )));
        assert!(!qualifies_for_honeypot(&truth("fact", 0.5, Some("wave_01_foo.md"))));
        assert!(!qualifies_for_honeypot(&truth("fact", 0.9, None)));
    }

    #[test]
    fn eligibility_reasons_are_structured() {
        assert_eq!(
            honeypot_eligibility(&truth("fact", 0.5, Some("wave.md"))).unwrap_err(),
            HoneypotRejectReason::LowConfidence {
                got: 0.5,
                min: HONEYPOT_MIN_CONFIDENCE,
            }
        );
        assert_eq!(
            honeypot_eligibility(&truth("fact", 0.9, None)).unwrap_err(),
            HoneypotRejectReason::MissingSourceFile
        );
    }

    #[test]
    fn threshold_is_080_not_085() {
        // 0.80 qualifies (tuned down from 0.85 on 2026-06-15)
        assert!(qualifies_for_honeypot(&truth(
            "[SYSTEM:Test] threshold test",
            0.80,
            Some("wave_01_test.md"),
        )));
        // 0.79 still rejected
        assert!(!qualifies_for_honeypot(&truth(
            "[SYSTEM:Test] below threshold",
            0.79,
            Some("wave_01_test.md"),
        )));
    }
}
