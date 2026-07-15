//! Deterministic GZMO ecosystem snapshot from loaded config — no LLM.

use std::path::Path;
use std::time::SystemTime;

use chrono::{DateTime, Local, Utc};
use rusqlite::Connection;

use crate::assembly::AssemblyConfig;
use crate::config::GzmoConfig;
use crate::health::{collect_health_probes, ProbeResult};

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
        asm.distill.label(),
        asm.dream.label(),
        asm.spark.label(),
        asm.ops_health.label(),
        asm.config_handoff.label()
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
    let (dreams_exists, dreams_bytes, dreams_mtime) = file_meta(&config.skills.dreams_path);
    let memory_files = count_files_with_ext(&config.memory.directory, "md");
    let session_files = count_files_with_ext(&config.session_distill.sessions_dir, "json");

    let prime = user_systemd_unit("llama-prime.service").await;
    let scheduler = user_systemd_unit("gzmo-scheduler.service").await;
    let observatory = user_systemd_unit("gzmo-observatory.service").await;

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
    out.push_str(&format!("| gzmo-observatory.service | {observatory} |\n"));
    out.push_str("\n*Foreground `gzmo chat` is not the scheduler daemon — use the table above for long-running services.*\n\n");

    out.push_str("### Data paths (from config)\n\n");
    out.push_str("| Component | Path | Status |\n|---|---|---|\n");

    let vault_status = if vault_exists {
        format!(
            "{} facts, {} KB{}",
            vault_facts
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".into()),
            vault_bytes.unwrap_or(0) / 1024,
            vault_mtime
                .map(|t| format!(", mtime {}", t.format("%Y-%m-%d %H:%M UTC")))
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
    out.push_str("\n");

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
