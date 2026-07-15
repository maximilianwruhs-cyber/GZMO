# Subsystem — Heartbeat & CheapCheck

**Source:** `gzmo-core/src/daemon.rs`, wired in `gzmo-cli/src/daemon_cmd.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Implements temporal autonomy with **CheapCheck** triage: run lightweight deterministic probes on an interval, and only invoke the expensive LLM pipeline when anomalies are detected. Also provides shared **cron catch-up** helpers used by dream, spark, distill, and Qdrant sync loops.

---

## 2. How it works

### CheapCheck trait

```65:75:gzmo-core/src/daemon.rs
/// A lightweight, deterministic check that runs WITHOUT invoking the LLM.
/// Implementations grep logs, ping health endpoints, check RSS feeds, etc.
/// Only if a CheapCheck returns Some(anomaly) does the system wake the GPU.
#[async_trait]
pub trait CheapCheck: Send + Sync {
    /// A human-readable name for this check (used in logs).
    fn name(&self) -> &str;

    /// Execute the check. Returns Some(description) if action is needed.
    async fn evaluate(&self) -> Result<Option<String>>;
}
```

### FileChangeCheck

Scans `watch_dir` for files modified within `since` duration:

```81:109:gzmo-core/src/daemon.rs
pub struct FileChangeCheck {
    pub watch_dir: String,
    pub since: Duration,
}

#[async_trait]
impl CheapCheck for FileChangeCheck {
    async fn evaluate(&self) -> Result<Option<String>> {
        let cutoff = std::time::SystemTime::now() - self.since;
        let mut entries = tokio::fs::read_dir(&self.watch_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            // ... modified > cutoff → anomaly
        }
        Ok(None)
    }
}
```

### HealthPing (stealth mode)

500ms timeout; suppresses timeout/connect errors to avoid recon noise:

```112:155:gzmo-core/src/daemon.rs
pub struct HealthPing {
    pub url: String,
    pub service_name: String,
}
// ...
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .build()?;
        // ...
                if e.is_timeout() || e.is_connect() {
                    debug!(service = %self.service_name, "Stealth probe suppressed network error (assumed offline)");
                    Ok(None)
```

### HeartbeatEngine loop

```203:222:gzmo-core/src/daemon.rs
    pub async fn run<F, Fut>(&self, on_anomalies: F) -> !
    where
        F: Fn(Vec<String>) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let mut interval = tokio::time::interval(self.interval);
        loop {
            interval.tick().await;
            let anomalies = self.tick().await;
            if !anomalies.is_empty() {
                info!(count = anomalies.len(), "Waking LLM for autonomous cycle");
                on_anomalies(anomalies).await;
            }
        }
    }
```

### Daemon wiring

```90:100:gzmo-cli/src/daemon_cmd.rs
    let heartbeat_interval = Duration::from_secs(config.agent.heartbeat_interval_secs);
    let mut heartbeat = HeartbeatEngine::new(heartbeat_interval);
    heartbeat.add_check(FileChangeCheck {
        watch_dir: config.memory.directory.to_string_lossy().to_string(),
        since: Duration::from_secs(config.agent.heartbeat_interval_secs),
    });
    heartbeat.add_check(HealthPing {
        url: format!("{}/models", config.engine.active_engine().url),
        service_name: "LLM Engine".to_string(),
    });
```

### Cron catch-up (shared)

```23:38:gzmo-core/src/daemon.rs
pub fn cron_due_today(
    now: &DateTime<Utc>,
    hour: u32,
    minute: u32,
    last_run_date: Option<NaiveDate>,
) -> bool {
    let today = now.date_naive();
    if last_run_date == Some(today) {
        return false;
    }
    let now_mins = now.hour() * 60 + now.minute();
    now_mins >= cron_minutes(hour, minute)
}
```

---

## 3. Interfaces

| Interface | Config / path |
|-----------|---------------|
| Heartbeat interval | `[agent] heartbeat_interval_secs` |
| Memory watch dir | `[memory] directory` (typically `data/` or `memory/`) |
| LLM ping URL | `{active_engine.url}/models` |
| Output artifact | `data/HEARTBEAT.md` (operator-facing) |
| Cron helpers | `cron_due_today`, `spark_cron_slot_due` — used by cognition loops |

---

## 4. THINKING nodes

> **THINKING — daemon.rs:HealthPing stealth**
> - *Reviewed:* Timeouts/connect failures return `Ok(None)`.
> - *Insight:* Deliberate EDR-stealth — offline LLM does not trigger anomaly storm.
> - *Risk / limitation:* True LLM outage may go unnoticed until scheduled cognition fails.
> - *Enhancement:* Separate strict probe for startup vs stealth heartbeat. [CT101-safe]

> **THINKING — daemon.rs:cron_due_today**
> - *Reviewed:* Fires on first tick after scheduled time, not exact minute match.
> - *Insight:* Daemon restart at 02:30 still runs 01:00 dream job same day.
> - *Risk / limitation:* Multiple slots same day need separate `last_run_date` keys (spark uses tuple).
> - *Enhancement:* Unified `CronJobState` struct in config/data dir. [GZMO-next]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| CT101 | Heartbeat logs anomalies; full LLM wake on anomaly is minimal today |
| GZMO-next | Expand CheapChecks: Qdrant lag, vault size, synapse tail age |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Append heartbeat results to `HEARTBEAT.md` table | [CT101-safe] |
| 2 | CheapCheck for sidecar TCP (Redis/Qdrant/Neo4j) | [CT101-safe] |
| 3 | Configurable stealth vs strict HealthPing | [CT101-safe] |
| 4 | Prometheus metrics for tick/anomaly counts | [GZMO-next] |
