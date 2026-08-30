//! Immune Patrol — contradiction hunt after dream promote.
//!
//! Default path is plan-only (dry_run). Bounded apply requires explicit confirm
//! (`IMMUNE_APPLY=1`) and writes an apply receipt for rollback.
//! Value-forgetting plans use SCM-style low-utility candidates.

use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::memory::vault::SqliteVault;
use crate::types::ExtractedTruth;

const SCHEMA: &str = "gzmo.immune.plan/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmuneCandidate {
    pub fact_id: String,
    pub content: String,
    pub reason: String,
    pub against_truth: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmunePlan {
    pub schema: String,
    pub night_id: String,
    pub generated_at: String,
    pub dry_run: bool,
    pub truths_scanned: usize,
    pub candidates: Vec<ImmuneCandidate>,
}

/// Accurate ops notes that must not be treated as stale engine lore.
fn is_exempt_ops_note(content: &str) -> bool {
    let c = content.to_lowercase();
    c.contains("legacy auto_dream")
        || c.contains("legacy auto-dream")
        || c.contains("[state:enginesenabled]")
        || c.contains("enabled on ct101 living stack")
        || c.contains("clean-slate rebuild complete")
        || c.contains("prior clean-slate")
        || c.contains("operator apply 2026-07-20")
}

fn mentions_clean_slate_disabled(content: &str) -> bool {
    let c = content.to_lowercase();
    let disabled = c.contains("currently disabled")
        || c.contains("noted as 'currently disabled")
        || c.contains("noted as \"currently disabled")
        || c.contains("enabled=false");
    let clean = c.contains("clean-slate") || c.contains("clean slate");
    disabled && clean
}

/// Stale DreamEngine / Dream schedule lore claiming the engine is off.
fn is_stale_dreamengine_disabled_lore(content: &str) -> bool {
    if is_exempt_ops_note(content) {
        return false;
    }
    let c = content.to_lowercase();
    let mentions_engine = c.contains("dreamengine") || c.contains("[system:dream]");
    mentions_engine && mentions_clean_slate_disabled(content)
}

/// Stale SparkEngine / Spark schedule lore.
fn is_stale_spark_disabled_lore(content: &str) -> bool {
    if is_exempt_ops_note(content) {
        return false;
    }
    let c = content.to_lowercase();
    let mentions =
        c.contains("sparkengine") || c.contains("[system:spark]") || c.contains("[systems:spark");
    mentions && mentions_clean_slate_disabled(content)
}

/// Stale SessionDistill lore (any casing).
fn is_stale_session_distill_disabled_lore(content: &str) -> bool {
    if is_exempt_ops_note(content) {
        return false;
    }
    let c = content.to_lowercase();
    let mentions = c.contains("sessiondistill") || c.contains("session_distill");
    mentions && mentions_clean_slate_disabled(content)
}

/// Meta claim that dreams/spark/session_distill are enabled=false for clean-slate.
fn is_stale_engines_disabled_state(content: &str) -> bool {
    if is_exempt_ops_note(content) {
        return false;
    }
    let c = content.to_lowercase();
    c.contains("[state:enginesdisabled]")
        || (c.contains("enabled=false")
            && (c.contains("[dreams]") || c.contains("[spark]") || c.contains("[session_distill]"))
            && (c.contains("clean-slate") || c.contains("clean slate")))
}

/// Global clean-slate disabled class (any of the engine families).
fn is_stale_clean_slate_engine_lore(content: &str) -> Option<&'static str> {
    if is_stale_engines_disabled_state(content) {
        return Some("global_engines_disabled_state");
    }
    if is_stale_dreamengine_disabled_lore(content) {
        return Some("global_dreamengine_disabled_lore");
    }
    if is_stale_spark_disabled_lore(content) {
        return Some("global_spark_disabled_lore");
    }
    if is_stale_session_distill_disabled_lore(content) {
        return Some("global_session_distill_disabled_lore");
    }
    None
}

/// Polarity / status tokens that often co-occur with superseded lore.
fn contradiction_reason(truth: &str, candidate: &str) -> Option<&'static str> {
    let t = truth.to_lowercase();
    if let Some(reason) = is_stale_clean_slate_engine_lore(candidate) {
        if t.contains("dream")
            || t.contains("consolidat")
            || t.contains("verified_dream")
            || t.contains("spark")
            || t.contains("session")
            || t.contains("distill")
            || t.contains("enabled")
        {
            return Some(reason);
        }
        // Still surface against empty/generic night truth via currently_disabled
        if !t.contains("disabled") {
            return Some(reason);
        }
    }
    let c = candidate.to_lowercase();
    if (c.contains("disabled") || c.contains("turned off") || c.contains("not running"))
        && (t.contains("enabled") || t.contains("running") || t.contains("active"))
        && !is_exempt_ops_note(candidate)
        && !c.contains("legacy auto_dream")
    {
        return Some("status_polarity_disabled_vs_active");
    }
    None
}

fn entity_needles(content: &str) -> Vec<String> {
    let mut needles = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find('[') {
        let after = &rest[start + 1..];
        if let Some(end) = after.find(']') {
            let tag = &after[..end];
            if let Some((_, name)) = tag.split_once(':') {
                let n = name.trim();
                if n.len() >= 3 {
                    needles.push(n.to_string());
                }
            }
            rest = &after[end + 1..];
        } else {
            break;
        }
    }
    for w in content.split(|c: char| !c.is_alphanumeric() && c != '_') {
        if w.len() >= 4
            && w.chars().next().is_some_and(|c| c.is_uppercase())
            && w.chars().any(|c| c.is_lowercase())
        {
            needles.push(w.to_string());
        }
    }
    needles.sort();
    needles.dedup();
    needles.into_iter().take(8).collect()
}

fn push_candidate(
    candidates: &mut Vec<ImmuneCandidate>,
    seen: &mut std::collections::HashSet<Uuid>,
    id: Uuid,
    content: String,
    reason: &str,
    against: String,
) {
    if !seen.insert(id) {
        return;
    }
    candidates.push(ImmuneCandidate {
        fact_id: id.to_string(),
        content: content.chars().take(400).collect(),
        reason: reason.to_string(),
        against_truth: against.chars().take(240).collect(),
        action: "tombstone_or_supersede".into(),
    });
}

fn global_clean_slate_patrol(
    vault: &SqliteVault,
    night: NaiveDate,
    candidates: &mut Vec<ImmuneCandidate>,
    seen: &mut std::collections::HashSet<Uuid>,
) {
    let patterns = [
        "%DreamEngine%",
        "%SparkEngine%",
        "%SessionDistill%",
        "%session_distill%",
        "%EnginesDisabled%",
        "%[SYSTEM:Spark]%",
        "%[SYSTEM:Dream]%",
    ];
    let against = format!("living metabolism active night={night}");
    for pat in patterns {
        let Ok(rows) = vault.honeypot_latest_matching(pat, 64) else {
            continue;
        };
        for (id, content, _) in rows {
            if let Some(reason) = is_stale_clean_slate_engine_lore(&content) {
                push_candidate(candidates, seen, id, content, reason, against.clone());
            }
        }
    }
}

/// Scan latest honeypot for facts that contradict tonight's dream truths.
/// Writes `{vault_parent}/immune/plan-{night}.json` + `latest.json`. Never applies.
pub fn run_patrol(
    vault: &SqliteVault,
    night: NaiveDate,
    truths: &[ExtractedTruth],
) -> Result<PathBuf> {
    let mut candidates: Vec<ImmuneCandidate> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for truth in truths {
        let needles = entity_needles(&truth.content);
        if needles.is_empty() {
            continue;
        }
        for needle in needles {
            let pattern = format!("%{needle}%");
            let rows = match vault.honeypot_latest_matching(&pattern, 16) {
                Ok(r) => r,
                Err(_) => continue,
            };
            for (id, content, _) in rows {
                if content == truth.content || seen.contains(&id) {
                    continue;
                }
                if let Some(reason) = contradiction_reason(&truth.content, &content) {
                    push_candidate(
                        &mut candidates,
                        &mut seen,
                        id,
                        content,
                        reason,
                        truth.content.clone(),
                    );
                }
            }
        }
    }

    global_clean_slate_patrol(vault, night, &mut candidates, &mut seen);

    let plan = ImmunePlan {
        schema: SCHEMA.into(),
        night_id: night.to_string(),
        generated_at: Utc::now().to_rfc3339(),
        dry_run: true,
        truths_scanned: truths.len(),
        candidates,
    };

    let dir = vault.data_dir().join("immune");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("plan-{night}.json"));
    let text = serde_json::to_string_pretty(&plan)? + "\n";
    std::fs::write(&path, &text)?;
    std::fs::write(dir.join("latest.json"), text)?;
    tracing::info!(
        path = %path.display(),
        candidates = plan.candidates.len(),
        "Immune patrol plan written (dry_run)"
    );
    Ok(path)
}

/// SCM-inspired value forgetting: low utility + never felt → tombstone candidates (plan only).
pub fn run_value_forgetting_plan(
    vault: &SqliteVault,
    night: NaiveDate,
    max_candidates: usize,
) -> Result<PathBuf> {
    let conn = vault.db_conn()?;
    let limit = max_candidates.max(1) as i64;
    let mut stmt = conn.prepare(
        "SELECT id, content, utility_score, recall_count, confidence
         FROM honeypot
         WHERE is_latest = 1
           AND utility_score < 1.0
           AND recall_count = 0
           AND confidence < 0.92
         ORDER BY utility_score ASC, confidence ASC
         LIMIT ?1",
    )?;
    let mut candidates = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for row in stmt.query_map(params![limit], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, f64>(4)?,
        ))
    })? {
        let Ok((id_str, content, util, recall, conf)) = row else {
            continue;
        };
        let Ok(id) = Uuid::parse_str(&id_str) else {
            continue;
        };
        push_candidate(
            &mut candidates,
            &mut seen,
            id,
            content,
            "value_forgetting_low_utility",
            format!("utility={util:.3} recall={recall} conf={conf:.3}"),
        );
    }
    let plan = ImmunePlan {
        schema: SCHEMA.into(),
        night_id: night.to_string(),
        generated_at: Utc::now().to_rfc3339(),
        dry_run: true,
        truths_scanned: 0,
        candidates,
    };
    let dir = vault.data_dir().join("immune");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("forget-{night}.json"));
    let text = serde_json::to_string_pretty(&plan)? + "\n";
    std::fs::write(&path, &text)?;
    std::fs::write(dir.join("latest-forget.json"), &text)?;
    Ok(path)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmuneApplyReport {
    pub schema: String,
    pub night_id: String,
    pub applied_at: String,
    pub dry_run: bool,
    pub applied: usize,
    pub capped: usize,
    pub fact_ids: Vec<String>,
    pub rollback_note: String,
}

const APPLY_SCHEMA: &str = "gzmo.immune.apply/v1";

/// Bounded apply: supersede plan candidates (raw plan kept; curated apply receipt written).
/// Never runs unless `confirm_apply` is true. Caps at `max_apply`.
pub fn apply_plan(
    vault: &SqliteVault,
    plan: &ImmunePlan,
    max_apply: usize,
    confirm_apply: bool,
) -> Result<PathBuf> {
    if plan.candidates.is_empty() {
        return Ok(PathBuf::new());
    }
    if !confirm_apply {
        anyhow::bail!("immune apply refused — pass confirm_apply / IMMUNE_APPLY=1");
    }
    let cap = max_apply.clamp(1, 50);
    let conn = vault.db_conn()?;
    let mut applied_ids = Vec::new();
    for c in plan.candidates.iter().take(cap) {
        crate::memory::lifecycle::supersede_honeypot(&conn, &c.fact_id)?;
        applied_ids.push(c.fact_id.clone());
    }
    let report = ImmuneApplyReport {
        schema: APPLY_SCHEMA.into(),
        night_id: plan.night_id.clone(),
        applied_at: Utc::now().to_rfc3339(),
        dry_run: false,
        applied: applied_ids.len(),
        capped: cap,
        fact_ids: applied_ids.clone(),
        rollback_note: format!(
            "Re-set is_latest=1 for fact_ids in this receipt if Keep-quality goes RED. count={}",
            applied_ids.len()
        ),
    };
    let dir = vault.data_dir().join("immune");
    std::fs::create_dir_all(&dir)?;
    let stamp = Utc::now().format("%Y%m%dT%H%M%SZ");
    let path = dir.join(format!("applied-{stamp}.json"));
    let text = serde_json::to_string_pretty(&report)? + "\n";
    std::fs::write(&path, &text)?;
    std::fs::write(dir.join("latest-apply.json"), text)?;
    Ok(path)
}

/// Artifact directory helper for tests / lymph.
pub fn immune_dir(vault_db: &Path) -> PathBuf {
    vault_db
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("immune")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_dreamengine_disabled_vs_dream_truth() {
        let reason = contradiction_reason(
            "Dream consolidation promoted verified_dream truths tonight",
            "[SYSTEM:DreamEngine] DreamEngine currently disabled during clean-slate rebuild",
        );
        assert_eq!(reason, Some("global_dreamengine_disabled_lore"));
    }

    #[test]
    fn entity_needles_extract_bracket_tags() {
        let n = entity_needles("[SYSTEM:GZMO] four memory layers");
        assert!(n.iter().any(|x| x == "GZMO"));
    }

    #[test]
    fn legacy_auto_dream_disabled_is_not_stale_engine_lore() {
        assert!(!is_stale_dreamengine_disabled_lore(
            "[SYSTEMS:DreamEngine] Legacy auto_dream orchestration job is disabled (was at 03:00)"
        ));
    }

    #[test]
    fn clean_slate_disabled_is_stale_engine_lore() {
        assert!(is_stale_dreamengine_disabled_lore(
            "[SYSTEM:DreamEngine] Currently disabled during clean-slate rebuild."
        ));
        assert!(is_stale_spark_disabled_lore(
            "[SYSTEM:Spark] Currently disabled during clean-slate rebuild"
        ));
        assert!(is_stale_session_distill_disabled_lore(
            "[SYSTEM:SessionDistill] Currently disabled during clean-slate rebuild."
        ));
        assert!(is_stale_engines_disabled_state(
            "[STATE:EnginesDisabled] During the clean-slate rebuild, [dreams]/[spark]/[session_distill] are enabled=false"
        ));
    }

    #[test]
    fn enabled_replacement_is_exempt() {
        assert!(is_stale_clean_slate_engine_lore(
            "[SYSTEM:DreamEngine] Enabled on CT101 living stack — Prior clean-slate \"disabled\" lore is superseded (operator apply 2026-07-20)."
        )
        .is_none());
        assert!(is_stale_clean_slate_engine_lore(
            "[STATE:EnginesEnabled] Clean-slate rebuild complete on CT101 living stack — [dreams] enabled"
        )
        .is_none());
    }

    #[test]
    fn apply_plan_noop_on_empty() {
        let vault = SqliteVault::open(":memory:").unwrap();
        let plan = ImmunePlan {
            schema: SCHEMA.into(),
            night_id: "2024-12-12".into(),
            generated_at: "test".into(),
            dry_run: true,
            truths_scanned: 0,
            candidates: vec![],
        };
        let res = apply_plan(&vault, &plan, 5, false).unwrap();
        assert_eq!(res, PathBuf::new());
    }

    #[test]
    fn apply_plan_respects_bounds() {
        let vault = SqliteVault::open(":memory:").unwrap();

        let plan = ImmunePlan {
            schema: SCHEMA.into(),
            night_id: "test".into(),
            generated_at: "test".into(),
            dry_run: true,
            truths_scanned: 0,
            candidates: vec![
                ImmuneCandidate {
                    fact_id: "fake-1".into(),
                    content: "test".into(),
                    reason: "test".into(),
                    against_truth: "test".into(),
                    action: "test".into(),
                },
                ImmuneCandidate {
                    fact_id: "fake-2".into(),
                    content: "test".into(),
                    reason: "test".into(),
                    against_truth: "test".into(),
                    action: "test".into(),
                },
            ],
        };
        let res = apply_plan(&vault, &plan, 1, true);
        assert!(res.is_err());
    }
}
