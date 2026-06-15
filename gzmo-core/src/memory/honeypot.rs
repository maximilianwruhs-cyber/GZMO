//! Curated honeypot layer — subset of vault facts eligible for Qdrant / recall (M2).

use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};

use crate::types::ExtractedTruth;

/// Minimum confidence aligned with `[ingest] min_confidence` and vault promote gate.
pub const HONEYPOT_MIN_CONFIDENCE: f32 = 0.85;

pub fn qualifies_for_honeypot(truth: &ExtractedTruth) -> bool {
    if truth.confidence < HONEYPOT_MIN_CONFIDENCE {
        return false;
    }
    let Some(sf) = &truth.source_file else {
        return false;
    };
    if sf.trim().is_empty() {
        return false;
    }
    let sf_lower = sf.to_lowercase();
    if sf_lower.contains("chat_history")
        || sf_lower.contains("chat_session")
        || sf_lower.contains("quelltext")
        || sf_lower.contains("sources")
    {
        return false;
    }
    let lower = truth.content.to_lowercase();
    if lower.starts_with("[relation:") {
        return false;
    }
    !is_boilerplate(&lower)
}

fn is_boilerplate(lower: &str) -> bool {
    lower.contains("sources do not contain")
        || lower.contains("migration_id")
        || lower.contains("takeout drive")
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
            ?1, ?2, ?3, ?4, ?5, ?6, 'fact', ?7, ?8, 1, ?9, ?10, ?11, 'obolus', ?12, 1, 0
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
            ?1, ?2, ?3, ?4, ?5, ?6, 'fact', 1, ?7, ?8, ?9, 'obolus', ?10, 1, 0
        )
        ON CONFLICT(id) DO UPDATE SET
            content = excluded.content,
            content_norm = excluded.content_norm,
            embedding = excluded.embedding,
            origin = excluded.origin,
            confidence = MAX(confidence, excluded.confidence),
            decay_class = excluded.decay_class,
            source_file = excluded.source_file,
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
}
