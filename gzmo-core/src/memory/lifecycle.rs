//! Honeypot/vault lifecycle: duplicate, extends, contradicts (MEMORY_ARCHITECTURE_SPEC §3).

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::memory::vault::normalize_truth_content;
use crate::types::ExtractedTruth;

/// How a new truth relates to an existing honeypot fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleKind {
    /// Same normalized content — corroborate only.
    Duplicate,
    /// Enrichment — both stay `is_latest`.
    Extends,
    /// Replacement — old `is_latest = 0`, new row links `update`.
    Contradicts,
    /// No cluster match — independent insert.
    Unrelated,
    /// Inferred — must not enter honeypot until verified.
    Derives,
}

impl LifecycleKind {
    pub fn graph_rel(self) -> Option<&'static str> {
        match self {
            Self::Extends => Some("extends"),
            Self::Contradicts => Some("update"),
            Self::Derives => Some("derives"),
            _ => None,
        }
    }
}

/// Parse `[TYPE:Name]` entity anchor from curated fact text.
pub fn extract_primary_entity(content: &str) -> Option<String> {
    let start = content.find('[')?;
    let rest = &content[start + 1..];
    let end = rest.find(']')?;
    let inner = &rest[..end];
    let (_tag, name) = inner.split_once(':')?;
    let name = name.trim();
    if name.len() >= 2 {
        Some(name.to_string())
    } else {
        None
    }
}

/// Rule-based relation between an existing fact and a candidate (no LLM).
pub fn classify_truth_pair(old_content: &str, new_content: &str) -> LifecycleKind {
    let old_n = normalize_truth_content(old_content);
    let new_n = normalize_truth_content(new_content);
    if old_n == new_n {
        return LifecycleKind::Duplicate;
    }

    if contradicts_heuristic(old_content, new_content) {
        return LifecycleKind::Contradicts;
    }

    if is_extension(&old_n, &new_n) {
        return LifecycleKind::Extends;
    }

    LifecycleKind::Unrelated
}

fn has_negation_shift(old_n: &str, new_n: &str) -> bool {
    const NEG: &[&str] = &[
        "not ", "no longer", "instead of", "replaced", "deprecated", "never ", "stopped ",
    ];
    let old_has = NEG.iter().any(|m| old_n.contains(m));
    let new_has = NEG.iter().any(|m| new_n.contains(m));
    new_has && !old_has
}

/// New fact enriches old (all old tokens appear in order; new is longer).
fn is_extension(old_n: &str, new_n: &str) -> bool {
    if new_n == old_n || has_negation_shift(old_n, new_n) {
        return false;
    }
    if new_n.contains(old_n) || old_n.contains(new_n) {
        return new_n.len() > old_n.len();
    }
    let old_tokens: Vec<&str> = old_n.split_whitespace().filter(|t| t.len() >= 3).collect();
    if old_tokens.len() < 2 {
        return false;
    }
    let mut pos = 0usize;
    for t in &old_tokens {
        let Some(idx) = new_n[pos..].find(t) else {
            return false;
        };
        pos += idx + t.len();
    }
    new_n.len() > old_n.len() + 8
}

fn contradicts_heuristic(old: &str, new: &str) -> bool {
    let old_n = normalize_truth_content(old);
    let new_n = normalize_truth_content(new);
    if is_extension(&old_n, &new_n) {
        return false;
    }

    let Some(old_ent) = extract_primary_entity(old) else {
        return false;
    };
    let Some(new_ent) = extract_primary_entity(new) else {
        return false;
    };
    if normalize_entity_key(&old_ent) != normalize_entity_key(&new_ent) {
        return false;
    }

    let new_l = new.to_lowercase();
    const NEG: &[&str] = &[
        "not ",
        "no longer",
        "instead of",
        "replaced",
        "deprecated",
        "stopped ",
        "never ",
        "removed ",
        "disabled ",
    ];
    if NEG.iter().any(|m| new_l.contains(m)) {
        return token_overlap(old, new) >= 0.25;
    }

    // Same entity, different role/object predicate (e.g. "is X" vs "is Y").
    if let (Some(old_pred), Some(new_pred)) = (predicate_tail(old), predicate_tail(new)) {
        if old_pred != new_pred && token_overlap(old, new) >= 0.2 {
            return true;
        }
    }

    false
}

fn normalize_entity_key(name: &str) -> String {
    name.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn predicate_tail(content: &str) -> Option<String> {
    let idx = content.find(']')?;
    let tail = content[idx + 1..].trim();
    if tail.len() < 8 {
        return None;
    }
    Some(normalize_truth_content(tail))
}

fn token_overlap(a: &str, b: &str) -> f64 {
    let a_l = a.to_lowercase();
    let b_l = b.to_lowercase();
    let ta: std::collections::HashSet<_> = a_l
        .split_whitespace()
        .filter(|w| w.len() >= 4)
        .collect();
    let tb: std::collections::HashSet<_> = b_l
        .split_whitespace()
        .filter(|w| w.len() >= 4)
        .collect();
    if ta.is_empty() || tb.is_empty() {
        return 0.0;
    }
    let inter = ta.intersection(&tb).count() as f64;
    let union = ta.union(&tb).count() as f64;
    inter / union
}

/// Derived-cognition rows (dream/spark/session distill) are held to a higher
/// confidence bar than direct ingest, UNLESS they carry localized evidence.
/// Covers both legacy (`dream`/`spark`) and current (`verified_*`/`session_distill`)
/// origin strings so the gate is not silently bypassed by a rename.
pub fn is_unverified_derived(truth: &ExtractedTruth, origin: &str) -> bool {
    if truth.content.to_lowercase().starts_with("[derives:") {
        return true;
    }
    let derived = matches!(
        origin,
        "dream" | "verified_dream" | "spark" | "verified_spark" | "session_distill"
    );
    if !derived {
        return false;
    }
    // A grounded (evidence-bearing) derived fact has passed verify + localization.
    let has_evidence = truth
        .evidence
        .as_ref()
        .map(|e| !e.evidence_text.trim().is_empty())
        .unwrap_or(false);
    !has_evidence && truth.confidence < 0.92
}

/// Latest honeypot row for the same entity anchor.
pub fn find_latest_honeypot_by_entity(
    conn: &Connection,
    entity: &str,
    container_tag: &str,
) -> anyhow::Result<Option<(String, String)>> {
    let pattern = format!("%{}%", entity.replace('%', ""));
    let tag_pattern = format!("%:{}]%", entity.replace('%', ""));
    let row = conn.query_row(
        "SELECT id, content FROM honeypot
         WHERE is_latest = 1 AND container_tag = ?1
           AND (content LIKE ?2 OR content LIKE ?3)
         ORDER BY promoted_at DESC
         LIMIT 1",
        params![container_tag, pattern, tag_pattern],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    );
    match row {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Mark prior honeypot rows non-latest (contradiction / update).
pub fn supersede_honeypot(conn: &Connection, old_id: &str) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "DELETE FROM honeypot_fts WHERE rowid IN (
            SELECT rowid FROM honeypot WHERE (id = ?1 OR vault_id = ?1) AND is_latest = 1
        )",
        params![old_id],
    )?;
    conn.execute(
        "UPDATE honeypot SET is_latest = 0 WHERE (id = ?1 OR vault_id = ?1) AND is_latest = 1",
        params![old_id],
    )?;
    tracing::info!(old_id, superseded_at = %now, "Honeypot fact superseded (lifecycle update)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_when_normalized_equal() {
        assert_eq!(
            classify_truth_pair(
                "[AGENT:Foo] does X",
                "[AGENT:Foo]  does   X"
            ),
            LifecycleKind::Duplicate
        );
    }

    #[test]
    fn extends_when_superset() {
        assert_eq!(
            classify_truth_pair(
                "[AGENT:Foo] runs backups",
                "[AGENT:Foo] runs nightly backups with ZFS"
            ),
            LifecycleKind::Extends
        );
    }

    #[test]
    fn contradicts_on_negation_same_entity() {
        assert_eq!(
            classify_truth_pair(
                "[AGENT:Firewall] manages rules on LXC101",
                "[AGENT:Firewall] no longer manages rules on LXC101"
            ),
            LifecycleKind::Contradicts
        );
    }

    #[test]
    fn extract_entity_from_tag() {
        assert_eq!(
            extract_primary_entity("[CONCEPT:GZMO] identity formula").as_deref(),
            Some("GZMO")
        );
    }

    fn derived_truth(conf: f32, with_evidence: bool) -> ExtractedTruth {
        use crate::types::{DecayClass, EvidenceSpan};
        ExtractedTruth {
            id: uuid::Uuid::new_v4(),
            content: "[CONCEPT:GZMO] is a memory system".to_string(),
            confidence: conf,
            mmr_score: 0.0,
            source_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
            decay_class: DecayClass::CuratedVault,
            source_file: Some("memory/2026-06-05.md".to_string()),
            evidence: with_evidence.then(|| EvidenceSpan {
                evidence_text: "GZMO is a memory system.".to_string(),
                quote_verifier: "GZMO is a memory system".to_string(),
                char_start: Some(0),
                char_end: Some(24),
            }),
        }
    }

    #[test]
    fn verified_dream_origin_is_covered_by_gate() {
        // Low-confidence derived fact without evidence is blocked, regardless of rename.
        assert!(is_unverified_derived(&derived_truth(0.86, false), "verified_dream"));
        assert!(is_unverified_derived(&derived_truth(0.86, false), "dream"));
        assert!(is_unverified_derived(&derived_truth(0.86, false), "session_distill"));
    }

    #[test]
    fn grounded_or_high_conf_derived_passes() {
        // Evidence-bearing derived fact passes even below 0.92.
        assert!(!is_unverified_derived(&derived_truth(0.86, true), "verified_dream"));
        // High-confidence derived fact passes.
        assert!(!is_unverified_derived(&derived_truth(0.95, false), "verified_dream"));
        // Direct ingest is never treated as derived.
        assert!(!is_unverified_derived(&derived_truth(0.50, false), "ingest"));
    }
}
