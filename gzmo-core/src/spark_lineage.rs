//! Operator spark lineage (Experience B) — normalize last SparkReport for status / Observatory.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde_json::Value;

/// Compact card for `gzmo status`, Observatory LED, and markdown digests.
#[derive(Debug, Clone, PartialEq)]
pub struct SparkLineageCard {
    pub path: PathBuf,
    pub date: String,
    pub anchor_preview: String,
    pub stale_sweetness: Option<f64>,
    pub promoted: Option<bool>,
    pub verdict_supported: Option<bool>,
    pub dry_run: Option<bool>,
    pub skip_reason: Option<String>,
    pub selection_score: Option<f64>,
    pub refractory_multiplier: Option<f64>,
    pub refractory_reason: Option<String>,
    pub soft_pick_top_k: Option<u64>,
    pub soft_pick_candidates: Option<u64>,
    pub updated_at: Option<String>,
    pub schema: Option<String>,
}

impl SparkLineageCard {
    /// Experience B honesty: selected anchor with mid-window stale_sweetness when present.
    pub fn experience_b_ok(&self) -> bool {
        !self.anchor_preview.is_empty()
            && self.anchor_preview != "(none)"
            && self.skip_reason.is_none()
            && self.stale_sweetness.map(|s| s > 0.0).unwrap_or(false)
    }

    pub fn observatory_detail(&self) -> String {
        let mut parts = vec![format!("date={}", self.date)];
        if let Some(s) = self.stale_sweetness {
            parts.push(format!("stale={s:.2}"));
        }
        match self.verdict_supported {
            Some(true) => parts.push("verdict=supported".into()),
            Some(false) => parts.push("verdict=unsupported".into()),
            None if self.dry_run == Some(true) => parts.push("dry_run".into()),
            None => {}
        }
        if let Some(p) = self.promoted {
            parts.push(format!("promoted={p}"));
        }
        let preview: String = self.anchor_preview.chars().take(42).collect();
        parts.push(preview);
        parts.join(" · ")
    }

    pub fn format_markdown(&self) -> String {
        let mut lines = vec![
            "# Last spark".into(),
            String::new(),
            format!("- **Date:** {}", self.date),
            format!(
                "- **Anchor:** {}",
                if self.anchor_preview.len() > 120 {
                    format!("{}…", &self.anchor_preview[..120])
                } else {
                    self.anchor_preview.clone()
                }
            ),
        ];
        if let Some(s) = self.stale_sweetness {
            lines.push(format!("- **stale_sweetness:** {s:.2}"));
        }
        if let Some(p) = self.promoted {
            lines.push(format!("- **promoted:** {p}"));
        }
        match self.verdict_supported {
            Some(true) => lines.push("- **Verdict:** supported".into()),
            Some(false) => lines.push("- **Verdict:** not supported".into()),
            None => {
                if let Some(ref skip) = self.skip_reason {
                    lines.push(format!("- **Skip:** {skip}"));
                } else if self.dry_run == Some(true) {
                    lines.push("- **Verdict:** (dry-run / none)".into());
                }
            }
        }
        if let Some(d) = self.dry_run {
            lines.push(format!("- **dry_run:** {d}"));
        }
        if let Some(m) = self.refractory_multiplier {
            let reason = self.refractory_reason.as_deref().unwrap_or("—");
            lines.push(format!("- **refractory:** ×{m:.3} ({reason})"));
        }
        if let Some(k) = self.soft_pick_top_k {
            let scored = self.soft_pick_candidates.unwrap_or(0);
            lines.push(format!("- **soft_pick:** top_k={k} candidates={scored}"));
        }
        if let Some(score) = self.selection_score {
            lines.push(format!("- **selection_score:** {score:.4}"));
        }
        if let Some(ref u) = self.updated_at {
            lines.push(format!("- **updated_at:** {u}"));
        }
        lines.push(format!("- **Path:** `{}`", self.path.display()));
        lines.push(String::new());
        lines.join("\n")
    }
}

/// Resolve `…/spark/last-spark-report.json` next to the vault db.
pub fn last_spark_report_path(vault_db: &Path) -> Option<PathBuf> {
    vault_db
        .parent()
        .map(|p| p.join("spark/last-spark-report.json"))
}

pub fn load_spark_lineage(vault_db: &Path) -> Option<SparkLineageCard> {
    let path = last_spark_report_path(vault_db)?;
    if !path.is_file() {
        return None;
    }
    let raw = std::fs::read_to_string(&path).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    Some(parse_spark_lineage(&path, &v))
}

pub fn parse_spark_lineage(path: &Path, v: &Value) -> SparkLineageCard {
    let anchor = v
        .pointer("/selection/anchor/content")
        .and_then(|x| x.as_str())
        .or_else(|| v.get("anchor_preview").and_then(|x| x.as_str()))
        .unwrap_or("(none)")
        .to_string();
    let stale = v
        .pointer("/selection/stale_sweetness")
        .and_then(|x| x.as_f64())
        .or_else(|| {
            // Recover from age when spark-link omitted the field (pre-O6 reports).
            let created = v
                .pointer("/selection/anchor/created_at")
                .and_then(|x| x.as_str())?;
            let created = DateTime::parse_from_rfc3339(created)
                .ok()?
                .with_timezone(&Utc);
            let date = v.get("date").and_then(|x| x.as_str()).unwrap_or("");
            let as_of = if date.len() >= 10 {
                DateTime::parse_from_rfc3339(&format!("{}T12:00:00Z", &date[..10]))
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now)
            } else {
                Utc::now()
            };
            let days = (as_of - created).num_days() as f64;
            Some(triangular_stale_sweetness(days, 14.0, 90.0))
        });
    SparkLineageCard {
        path: path.to_path_buf(),
        date: v
            .get("date")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_string(),
        anchor_preview: anchor,
        stale_sweetness: stale,
        promoted: v.get("promoted").and_then(|x| x.as_bool()),
        verdict_supported: v.pointer("/verdict/supported").and_then(|x| x.as_bool()),
        dry_run: v.get("dry_run").and_then(|x| x.as_bool()),
        skip_reason: v
            .get("skip_reason")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        selection_score: v
            .get("selection_score")
            .and_then(|x| x.as_f64())
            .or_else(|| {
                v.pointer("/selection/anchor_score")
                    .and_then(|x| x.as_f64())
            }),
        refractory_multiplier: v.pointer("/refractory/multiplier").and_then(|x| x.as_f64()),
        refractory_reason: v
            .pointer("/refractory/reason")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        soft_pick_top_k: v.pointer("/soft_pick/top_k").and_then(|x| x.as_u64()),
        soft_pick_candidates: v
            .pointer("/soft_pick/candidates_scored")
            .and_then(|x| x.as_u64()),
        updated_at: v
            .get("updated_at")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        schema: v
            .get("schema")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
    }
}

/// Same triangle as spark-link `stale_sweetness` (peak at mid window).
pub fn triangular_stale_sweetness(days: f64, min_days: f64, max_days: f64) -> f64 {
    if days < min_days || days > max_days || max_days <= min_days {
        return 0.0;
    }
    let mid = (min_days + max_days) / 2.0;
    let half = (max_days - min_days) / 2.0;
    if half <= 0.0 {
        return 0.0;
    }
    1.0 - ((days - mid).abs() / half)
}

/// Write `latest-card.md` + `lineage-latest.json` beside a spark report.
pub fn write_lineage_artifacts(spark_dir: &Path, card: &SparkLineageCard) -> std::io::Result<()> {
    std::fs::create_dir_all(spark_dir)?;
    std::fs::write(spark_dir.join("latest-card.md"), card.format_markdown())?;
    let json = serde_json::json!({
        "schema": "gzmo.spark.lineage_card/v1",
        "experience_b_ok": card.experience_b_ok(),
        "date": card.date,
        "anchor_preview": card.anchor_preview,
        "stale_sweetness": card.stale_sweetness,
        "promoted": card.promoted,
        "verdict_supported": card.verdict_supported,
        "dry_run": card.dry_run,
        "skip_reason": card.skip_reason,
        "selection_score": card.selection_score,
        "refractory_multiplier": card.refractory_multiplier,
        "refractory_reason": card.refractory_reason,
        "report_path": card.path.display().to_string(),
        "updated_at": Utc::now().to_rfc3339(),
    });
    std::fs::write(
        spark_dir.join("lineage-latest.json"),
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".into()) + "\n",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_spark_link_selection_with_stale() {
        let v = json!({
            "date": "2026-07-22",
            "promoted": false,
            "dry_run": true,
            "selection": {
                "anchor": {"content": "[CONCEPT:spark] mid-window anchor"},
                "stale_sweetness": 0.72,
                "anchor_score": 0.8
            }
        });
        let card = parse_spark_lineage(Path::new("/tmp/last-spark-report.json"), &v);
        assert!(card.experience_b_ok());
        assert!((card.stale_sweetness.unwrap() - 0.72).abs() < 1e-9);
        assert!(card.format_markdown().contains("stale_sweetness"));
    }

    #[test]
    fn recovers_stale_from_created_at() {
        let v = json!({
            "date": "2026-07-22",
            "selection": {
                "anchor": {
                    "content": "[CONCEPT:x] aged",
                    "created_at": "2026-05-25T08:00:00Z"
                }
            }
        });
        let card = parse_spark_lineage(Path::new("/tmp/r.json"), &v);
        assert!(card.stale_sweetness.unwrap() > 0.0);
    }

    #[test]
    fn triangle_peaks_mid_window() {
        let mid = triangular_stale_sweetness(52.0, 14.0, 90.0);
        let edge = triangular_stale_sweetness(14.0, 14.0, 90.0);
        assert!(mid > edge);
        assert!(mid > 0.9);
    }
}
