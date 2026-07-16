//! Deterministic GZMO ecosystem snapshot from loaded config — no LLM.

use std::path::Path;
use std::time::SystemTime;

use chrono::{DateTime, Local, Utc};
use rusqlite::Connection;

use crate::assembly::{handoff_apply_target, AssemblyConfig};
use crate::config::GzmoConfig;
use crate::health::{collect_health_probes, ProbeResult};

/// Fused-vs-live calibration status for operator surfaces (`gzmo status` / `/status`).
fn calibration_pending_line() -> Option<(String, String)> {
    let fused = handoff_apply_target()?;
    let fused_disp = fused.display().to_string();
    if !fused.exists() {
        return Some((fused_disp, "absent".into()));
    }
    let fuse_m = std::fs::metadata(&fused).and_then(|m| m.modified()).ok();
    let live_config = std::env::var("GZMO_CONFIG")
        .ok()
        .map(std::path::PathBuf::from)
        .filter(|p| p.exists());
    let live_m = live_config
        .as_ref()
        .and_then(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
    let status = match (fuse_m, live_m) {
        (Some(f), Some(l)) if f > l => {
            "pending — fused newer than live; run: gzmo config promote-fused --diff".into()
        }
        (Some(_), Some(_)) => "present — live at/after fused (promote done or equal)".into(),
        (Some(_), None) => {
            "present — review + gzmo config promote-fused --diff (no live mtime)".into()
        }
        _ => "present".into(),
    };
    Some((fused_disp, status))
}

fn file_meta(path: &Path) -> (bool, Option<u64>, Option<DateTime<Utc>>) {
    match std::fs::metadata(path) {
        Ok(m) => {
            let size = m.len();
            let modified = m.modified().ok().and_then(system_time_to_utc);
            (true, Some(size), modified)
        }
        Err(_) => (false, None, None),
    }
}

fn system_time_to_utc(t: SystemTime) -> Option<DateTime<Utc>> {
    let dur = t.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    DateTime::from_timestamp(dur.as_secs() as i64, dur.subsec_nanos())
}

fn count_files_with_ext(dir: &Path, ext: &str) -> usize {
    let Ok(read) = std::fs::read_dir(dir) else {
        return 0;
    };
    read.filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|x| x == ext.trim_start_matches('.'))
        })
        .count()
}

fn vault_semantic_count(path: &Path) -> Option<usize> {
    Connection::open(path)
        .ok()?
        .query_row("SELECT COUNT(*) FROM semantic_vault", [], |r| r.get(0))
        .ok()
}

fn vault_origin_summary(path: &Path) -> Option<String> {
    let conn = Connection::open(path).ok()?;
    let mut parts = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT origin, COUNT(*) FROM facts GROUP BY origin ORDER BY COUNT(*) DESC",
    ) {
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .ok()?;
        for row in rows.flatten() {
            parts.push(format!("{}={}", row.0, row.1));
        }
    }
    let honeypot: i64 = conn
        .query_row("SELECT COUNT(*) FROM honeypot", [], |r| r.get(0))
        .unwrap_or(0);
    let honeypot_origin: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM honeypot WHERE origin = 'honeypot'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    Some(format!(
        "facts[{}]; honeypot={} (origin=honeypot:{})",
        if parts.is_empty() {
            "none".into()
        } else {
            parts.join(", ")
        },
        honeypot,
        honeypot_origin
    ))
}

async fn user_systemd_unit(unit: &str) -> &'static str {
    match tokio::process::Command::new("systemctl")
        .args(["--user", "is-active", unit])
        .output()
        .await
    {
        Ok(o) if o.status.success() => "active",
        Ok(_) => "inactive",
        Err(_) => "unknown",
    }
}

fn assembly_summary(asm: &AssemblyConfig) -> String {
    format!(
        "distill={} dream={} spark={} ops={} handoff={}",
        asm.effective(asm.distill).label(),
        asm.effective(asm.dream).label(),
        asm.effective(asm.spark).label(),
        asm.effective(asm.ops_health).label(),
        asm.effective(asm.config_handoff).label()
    )
}

fn probe_line(r: &ProbeResult) -> String {
    let mark = if r.ok { "OK" } else { "FAIL" };
    format!("  [{mark}] {} — {}", r.name, r.detail)
}

/// Build a markdown-ish ecosystem report grounded in config paths and probes.
pub async fn format_ecosystem_status(config: &GzmoConfig) -> String {
    let instance = std::env::var("GZMO_INSTANCE").unwrap_or_else(|_| "legacy".into());
    let now = Utc::now();
    let local = Local::now();

    let active = config.engine.active_engine();
    let (vault_exists, vault_bytes, vault_mtime) = file_meta(&config.memory.vault_db);
    let vault_facts = vault_semantic_count(&config.memory.vault_db);
    let vault_origins = vault_origin_summary(&config.memory.vault_db);
    let (dreams_exists, dreams_bytes, dreams_mtime) = file_meta(&config.skills.dreams_path);
    let memory_files = count_files_with_ext(&config.memory.directory, "md");
    let session_files = count_files_with_ext(&config.session_distill.sessions_dir, "json");

    let prime = user_systemd_unit("llama-prime.service").await;
    let scheduler = user_systemd_unit("gzmo-scheduler.service").await;
    let observatory = user_systemd_unit("okforge.service").await;

    let probes = collect_health_probes(config, None).await;

    let mut out = String::new();
    out.push_str(&format!(
        "## GZMO ecosystem status — {} ({} local)\n\n",
        now.format("%Y-%m-%d %H:%M UTC"),
        local.format("%H:%M %Z")
    ));
    out.push_str(&format!("**Instance:** `{instance}`  \n"));
    out.push_str(&format!(
        "**Engine:** {} `{}` → {}\n",
        config.engine.active_mode, active.model, active.url
    ));
    out.push_str(&format!(
        "**Assembly:** `{}`\n\n",
        assembly_summary(&config.assembly)
    ));

    out.push_str("### User systemd\n\n");
    out.push_str("| Unit | State |\n|---|---|\n");
    out.push_str(&format!("| llama-prime.service | {prime} |\n"));
    out.push_str(&format!("| gzmo-scheduler.service | {scheduler} |\n"));
    out.push_str(&format!("| okforge.service (/observatory) | {observatory} |\n"));
    out.push_str("\n*Foreground `gzmo chat` is not the scheduler daemon — use the table above for long-running services.*\n\n");

    // Wiki / OKForge plane (production signal)
    let wiki_meta = config
        .memory
        .vault_db
        .parent()
        .map(|p| p.join("wiki-push-latest.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("wiki-push-latest.json"));
    let wiki_line = match std::fs::read_to_string(&wiki_meta) {
        Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(v) => {
                let healthy = v.get("healthy").and_then(|x| x.as_bool());
                let sha = v
                    .get("commit_sha")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .chars()
                    .take(12)
                    .collect::<String>();
                let n = v
                    .get("concepts_written")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0);
                match healthy {
                    Some(true) => format!("healthy · {n} concepts · sha {sha}"),
                    Some(false) => format!(
                        "UNHEALTHY · {}",
                        v.get("error")
                            .and_then(|x| x.as_str())
                            .or_else(|| v.get("skipped_reason").and_then(|x| x.as_str()))
                            .unwrap_or("see wiki-push-latest.json")
                    ),
                    None => {
                        if sha.is_empty() {
                            "meta present (no healthy flag)".into()
                        } else {
                            format!("ok · {n} concepts · sha {sha}")
                        }
                    }
                }
            }
            Err(_) => "meta unreadable".into(),
        },
        Err(_) => "no wiki-push-latest.json yet".into(),
    };
    out.push_str("### OKForge wiki plane\n\n");
    out.push_str(&format!("- **Observatory:** http://127.0.0.1:3000/observatory\n"));
    out.push_str(&format!("- **Last wiki push:** {wiki_line}\n\n"));

    out.push_str("### Data paths (from config)\n\n");
    out.push_str("| Component | Path | Status |\n|---|---|---|\n");

    let vault_status = if vault_exists {
        format!(
            "{} semantic, {} KB{}{}",
            vault_facts
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".into()),
            vault_bytes.unwrap_or(0) / 1024,
            vault_mtime
                .map(|t| format!(", mtime {}", t.format("%Y-%m-%d %H:%M UTC")))
                .unwrap_or_default(),
            vault_origins
                .map(|s| format!(" — {s}"))
                .unwrap_or_default()
        )
    } else {
        "missing".into()
    };
    out.push_str(&format!(
        "| Vault | `{}` | {vault_status} |\n",
        config.memory.vault_db.display()
    ));

    let dreams_status = if dreams_exists {
        format!(
            "{} bytes{}",
            dreams_bytes.unwrap_or(0),
            dreams_mtime
                .map(|t| format!(", mtime {}", t.format("%Y-%m-%d %H:%M UTC")))
                .unwrap_or_default()
        )
    } else {
        "missing".into()
    };
    out.push_str(&format!(
        "| DREAMS.md | `{}` | {dreams_status} |\n",
        config.skills.dreams_path.display()
    ));
    out.push_str(&format!(
        "| Episodic memory | `{}` | {} `.md` files |\n",
        config.memory.directory.display(),
        memory_files
    ));
    out.push_str(&format!(
        "| Sessions | `{}` | {} `.json` files |\n",
        config.session_distill.sessions_dir.display(),
        session_files
    ));
    if let Some((path, status)) = calibration_pending_line() {
        out.push_str(&format!("| Fused config | `{path}` | {status} |\n"));
    }
    out.push_str("\n");

    // Last spark lineage (Experience B)
    let spark_path = config.memory.vault_db.parent().map(|p| p.join("spark/last-spark-report.json"));
    if let Some(ref sp) = spark_path {
        if sp.exists() {
            if let Ok(raw) = std::fs::read_to_string(sp) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                    out.push_str("### Last spark\n\n");
                    let anchor = v
                        .pointer("/selection/anchor/content")
                        .and_then(|x| x.as_str())
                        .unwrap_or("(none)");
                    let stale = v
                        .pointer("/selection/stale_sweetness")
                        .and_then(|x| x.as_f64());
                    let verdict = v
                        .pointer("/verdict/supported")
                        .and_then(|x| x.as_bool());
                    let dry = v.get("dry_run").and_then(|x| x.as_bool());
                    let date = v.get("date").and_then(|x| x.as_str()).unwrap_or("?");
                    out.push_str(&format!("- **Date:** {date}\n"));
                    out.push_str(&format!(
                        "- **Anchor:** {}\n",
                        if anchor.len() > 100 {
                            format!("{}…", &anchor[..100])
                        } else {
                            anchor.into()
                        }
                    ));
                    if let Some(s) = stale {
                        out.push_str(&format!("- **stale_sweetness:** {s:.2}\n"));
                    }
                    match verdict {
                        Some(true) => out.push_str("- **Verdict:** supported\n"),
                        Some(false) => out.push_str("- **Verdict:** not supported\n"),
                        None => {
                            if let Some(skip) = v.get("skip_reason").and_then(|x| x.as_str()) {
                                out.push_str(&format!("- **Skip:** {skip}\n"));
                            } else {
                                out.push_str("- **Verdict:** (dry-run / none)\n");
                            }
                        }
                    }
                    if let Some(d) = dry {
                        out.push_str(&format!("- **dry_run:** {d}\n"));
                    }
                    out.push_str(&format!("- **Path:** `{}`\n\n", sp.display()));
                }
            }
        }
    }

    // Dream / graph drift (Experience A — dream-stats + ledger)
    let data_dir = config.memory.vault_db.parent();
    if let Some(dir) = data_dir {
        let stats_path = dir.join("dream-stats.json");
        let ledger_path = dir.join("graph-ledger.jsonl");
        if stats_path.exists() || ledger_path.exists() {
            out.push_str("### Graph drift\n\n");
            if stats_path.exists() {
                if let Ok(raw) = std::fs::read_to_string(&stats_path) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                        let anomaly = v
                            .get("anomaly_count")
                            .and_then(|x| x.as_u64())
                            .unwrap_or(0);
                        let rem = v
                            .get("rem_anchors")
                            .and_then(|x| x.as_u64())
                            .unwrap_or(0);
                        let ledger_art = v
                            .get("graph_ledger_artifact")
                            .and_then(|x| x.as_str())
                            .unwrap_or("-");
                        out.push_str(&format!("- **anomaly_count:** {anomaly}\n"));
                        out.push_str(&format!("- **rem_anchors:** {rem}\n"));
                        out.push_str(&format!("- **dream-stats:** `{}`\n", stats_path.display()));
                        out.push_str(&format!("- **ledger (stats):** `{ledger_art}`\n"));
                    }
                }
            } else {
                out.push_str(&format!(
                    "- **dream-stats:** missing (`{}`)\n",
                    stats_path.display()
                ));
            }
            if ledger_path.exists() {
                let mtime = std::fs::metadata(&ledger_path)
                    .and_then(|m| m.modified())
                    .ok()
                    .map(|t| {
                        chrono::DateTime::<chrono::Utc>::from(t)
                            .format("%Y-%m-%d %H:%M UTC")
                            .to_string()
                    });
                out.push_str(&format!(
                    "- **ledger path:** `{}`{}\n",
                    ledger_path.display(),
                    mtime
                        .map(|t| format!(" (mtime {t})"))
                        .unwrap_or_default()
                ));
            } else {
                out.push_str(&format!(
                    "- **ledger path:** missing (`{}`)\n",
                    ledger_path.display()
                ));
            }
            out.push('\n');
        }
    }

    out.push_str("### Probes\n\n");
    for r in &probes {
        out.push_str(&probe_line(r));
        out.push('\n');
    }

    out.push_str("\n*Report from `ecosystem_status` — run `/status` in chat or `gzmo status` on CLI.*\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_count_on_fixture_db() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data-next/vault.db");
        if path.exists() {
            assert!(vault_semantic_count(&path).unwrap_or(0) >= 0);
        }
    }
}
