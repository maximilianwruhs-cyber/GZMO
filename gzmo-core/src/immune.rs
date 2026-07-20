//! Immune Patrol — plan-only contradiction hunt after dream promote.
//!
//! Never mutates the living vault. Emits a supersession *plan* operators (or
//! forget-lint apply) can review. Self-development: the night that writes also
//! hunts its own stale dogma.

use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

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

/// True stale DreamEngine lore: claims the engine itself is off / clean-slate.
/// Excludes accurate notes that only the *legacy auto_dream job* is disabled.
fn is_stale_dreamengine_disabled_lore(content: &str) -> bool {
    let c = content.to_lowercase();
    if c.contains("legacy auto_dream") || c.contains("legacy auto-dream") {
        return false;
    }
    let mentions_engine = c.contains("dreamengine") || c.contains("[system:dream]");
    let stale =
        c.contains("currently disabled") || c.contains("clean-slate") || c.contains("clean slate");
    mentions_engine && stale
}

/// Polarity / status tokens that often co-occur with superseded lore.
fn contradiction_reason(truth: &str, candidate: &str) -> Option<&'static str> {
    let t = truth.to_lowercase();
    let c = candidate.to_lowercase();
    if is_stale_dreamengine_disabled_lore(candidate)
        && (t.contains("dream") || t.contains("consolidat") || t.contains("verified_dream"))
    {
        return Some("stale_dreamengine_disabled_while_dream_promotes");
    }
    if (c.contains("disabled") || c.contains("turned off") || c.contains("not running"))
        && (t.contains("enabled") || t.contains("running") || t.contains("active"))
        && !c.contains("legacy auto_dream")
    {
        return Some("status_polarity_disabled_vs_active");
    }
    if (c.contains("disabled") || c.contains("not enabled"))
        && (t.contains("promoted") || t.contains("consolidate") || t.contains("spark"))
        && is_stale_dreamengine_disabled_lore(candidate)
    {
        return Some("status_disabled_vs_overnight_activity");
    }
    if is_stale_dreamengine_disabled_lore(candidate) && !t.contains("disabled") {
        return Some("currently_disabled_claim");
    }
    None
}

fn entity_needles(content: &str) -> Vec<String> {
    let mut needles = Vec::new();
    // Bracket tags: [SYSTEM:GZMO], [TOOL:…], [PEOPLE:…]
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
    // Capitalized tokens (cheap entity hint)
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
            // Still check global dream-engine stale class against any dream night
            if let Ok(rows) = vault.honeypot_latest_matching("%DreamEngine%", 12) {
                for (id, content, _) in rows {
                    if seen.contains(&id) {
                        continue;
                    }
                    if let Some(reason) = contradiction_reason(&truth.content, &content) {
                        seen.insert(id);
                        candidates.push(ImmuneCandidate {
                            fact_id: id.to_string(),
                            content: content.chars().take(400).collect(),
                            reason: reason.to_string(),
                            against_truth: truth.content.chars().take(240).collect(),
                            action: "tombstone_or_supersede".into(),
                        });
                    }
                }
            }
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
                    seen.insert(id);
                    candidates.push(ImmuneCandidate {
                        fact_id: id.to_string(),
                        content: content.chars().take(400).collect(),
                        reason: reason.to_string(),
                        against_truth: truth.content.chars().take(240).collect(),
                        action: "tombstone_or_supersede".into(),
                    });
                }
            }
        }
    }

    // Global stale DreamEngine lore patrol (even if truths lack the name).
    // Broad LIKE then filter — avoids pulling "Legacy auto_dream … disabled" ops notes.
    if let Ok(rows) = vault.honeypot_latest_matching("%DreamEngine%", 48) {
        for (id, content, _) in rows {
            if seen.contains(&id) || !is_stale_dreamengine_disabled_lore(&content) {
                continue;
            }
            seen.insert(id);
            candidates.push(ImmuneCandidate {
                fact_id: id.to_string(),
                content: content.chars().take(400).collect(),
                reason: "global_dreamengine_disabled_lore".into(),
                against_truth: format!("dream consolidate night={night}"),
                action: "tombstone_or_supersede".into(),
            });
        }
    }

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
        assert_eq!(
            reason,
            Some("stale_dreamengine_disabled_while_dream_promotes")
        );
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
        assert!(is_stale_dreamengine_disabled_lore(
            "[SYSTEM:Dream] DreamEngine consolidates logs. Currently disabled during clean-slate rebuild."
        ));
    }
}
