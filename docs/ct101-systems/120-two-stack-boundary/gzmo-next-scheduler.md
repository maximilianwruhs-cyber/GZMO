# GZMO-next Scheduler — Thin Lab Cron Runner

**Source:** `gzmo-scheduler/src/*.rs`  
**Parent:** [120-two-stack-boundary/SYSTEM.md](./SYSTEM.md)

---

## Capability

**GZMO-next runtime process** — replaces fat `gzmo-daemon` on the workstation for the lab stack. Loads `gzmo-next.toml`, ticks every 60 s, and spawns Little Tools Lab recipe scripts for dream, distill, spark, ops health, and config handoff. **No** inline engines, vault, LLM client, or MCP in this crate.

---

## How it works

### Startup guards

```47:49:github-clone/GZMO/gzmo-scheduler/src/main.rs
    if std::env::var("GZMO_INSTANCE").as_deref() != Ok("next") {
        bail!("gzmo-scheduler is the GZMO-next runner: set GZMO_INSTANCE=next");
    }
```

```182:201:github-clone/GZMO/gzmo-scheduler/src/config.rs
    fn assert_all_lab(&self) -> Result<()> {
        let loops = [
            ("distill", a.distill),
            ("dream", a.dream),
            ("spark", a.spark),
            ("ops_health", a.ops_health),
            ("config_handoff", a.config_handoff),
        ];
        let inline: Vec<&str> = loops
            .iter()
            .filter(|(_, b)| *b == Backend::Inline)
            .map(|(name, _)| *name)
            .collect();
        if !inline.is_empty() {
            bail!(
                "gzmo-scheduler requires [assembly] all-lab; inline loops found: {}. \
                 Use `gzmo daemon` for inline engines.",
                inline.join(", ")
            );
        }
        Ok(())
    }
```

Symmetric opposite of CT101: scheduler **refuses** inline backends.

### Cron loop

```85:147:github-clone/GZMO/gzmo-scheduler/src/main.rs
async fn run_loop(cfg: &SchedulerConfig, config_path: &std::path::Path) -> Result<()> {
    run_job("ops_health", script, args).await;  // startup one-shot
    // ...
    loop {
        interval.tick().await;  // 60s
        // dream @ cfg.dreams.cron_hour:minute
        // distill @ session_distill cron
        // spark @ multi-slot cron_hours
        // config_handoff @ 04:00 UTC
    }
}
```

Uses catch-up cron (`cron_due_today`) — fires after restart if slot missed (same semantics as CT101 daemon).

### Job → recipe mapping

```10:48:github-clone/GZMO/gzmo-scheduler/src/jobs.rs
pub fn ops_args() -> (&'static str, Vec<String>) {
    ("ops-smoke.sh", vec!["--live".into()])
}
pub fn dream_args(cfg: &SchedulerConfig) -> (&'static str, Vec<String>) {
    ("session-to-dream.sh", vec!["--live".into(), "--output".into(), cfg.skills.dreams_path ...])
}
pub fn distill_args() -> (&'static str, Vec<String>) {
    ("synapse-distill-handoff.sh", vec!["--live".into()])
}
pub fn spark_args(cfg: &SchedulerConfig) -> (&'static str, Vec<String>) {
    ("cognition-smoke.sh", vec!["--live".into(), "--vault".into(), cfg.memory.vault_db ...])
}
pub fn handoff_args(config_path: &Path) -> (&'static str, Vec<String>) {
    ("gzmo-handoff.sh", vec!["--live".into(), "--apply".into(), "--gzmo-config".into(), fused_target])
}
```

### Subprocess spawn

```17:35:github-clone/GZMO/gzmo-scheduler/src/spawn.rs
pub async fn run_lab_script(script: &str, args: &[String]) -> Result<()> {
    let path = lab_root().join("scripts").join(script);
    let status = Command::new("bash")
        .arg(&path)
        .args(args)
        .status()
        .await?;
    // ...
}
```

Inherits scheduler env: `GZMO_CONFIG`, `GZMO_INSTANCE`, `LLM_URL`, `CARGO_TARGET_DIR`, etc.

### Default schedule (UTC)

| Job | Default cron |
|-----|--------------|
| ops_health | startup + recipe |
| dream | 01:00 |
| distill | 02:15 |
| spark | 03:30, 22:30 |
| config_handoff | 04:00 |

---

## Interfaces

| Interface | GZMO-next value |
|-----------|-----------------|
| Binary | `gzmo-scheduler` (workstation build) |
| PID lock | `/tmp/gzmo-scheduler.pid` (separate from `/tmp/gzmo_rust.pid`) |
| Config | `GZMO_CONFIG` → `gzmo-next.toml` |
| Vault | `data-next/vault.db` (resolved relative to config dir) |
| Dreams output | `DREAMS.md` path from `[skills]` |
| Lab root | `LITTLE_TOOLS_LAB_ROOT` or `github-clone/little-tools-lab` |

---

## THINKING nodes

> **THINKING — scheduler:assert_all_lab**
> - *Reviewed:* Refuses start if any `[assembly]` loop is inline.
> - *Insight:* Prevents accidentally pointing scheduler at CT101 legacy config.
> - *Risk / limitation:* Operator must maintain two config files — drift risk.
> - *Enhancement:* `gzmo config diff legacy next` command. [GZMO-next]

> **THINKING — scheduler:thin crate boundary**
> - *Reviewed:* No gzmo-core engine dependency — only TOML slices + bash spawn.
> - *Insight:* Fast compile, clear seam — cognitive logic lives in lab pieces.
> - *Risk / limitation:* Recipe failures only visible in stderr logs — no Synapse integration.
> - *Enhancement:* Structured job result JSONL per run. [GZMO-next]

> **THINKING — scheduler:handoff slot**
> - *Reviewed:* Config handoff runs daily; gate-fail exits non-zero but slot still consumed.
> - *Insight:* Prevents thrashing on bad calibration — holds previous config until next day.
> - *Risk / limitation:* Silent miss if operator doesn't read logs.
> - *Enhancement:* Observatory panel for last handoff status. [GZMO-next]

---

## Advancement

| CT101 `gzmo-daemon` | `gzmo-scheduler` |
|---------------------|------------------|
| Inline DreamEngine | `session-to-dream.sh --live` |
| Inline SessionDistill | `synapse-distill-handoff.sh` |
| Inline SparkEngine | `cognition-smoke.sh --spark-run` |
| 8 GiB RSS monolith | Thin cron parent + ephemeral recipe processes |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Job result JSONL log with duration/exit code | [GZMO-next] |
| 2 | `gzmo config diff` legacy vs next | [GZMO-next] |
| 3 | Synapse events for scheduler job lifecycle | [GZMO-next] |
| 4 | Health HTTP endpoint on scheduler for Observatory | [GZMO-next] |
| 5 | Align spark cron slots exactly with CT101 for beat-gate parity | [GZMO-next] |
