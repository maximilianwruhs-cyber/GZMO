# Subsystem — Spark Engine

**Sources:** `gzmo-core/src/spark.rs`, `gzmo-core/src/spark_schedule.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Chaos-free serendipitous recall: select a **stale** vault anchor fact, connect to **recent** context via LLM hypothesis, verify the link, and promote only `HYPOTHESIZED_LINK` (L3) — never new L2 facts at confidence 1.0. Appends verified sections to `DREAMS.md`.

---

## 2. How it works

### Engine + dual gateway

```57:108:gzmo-core/src/spark.rs
pub struct SparkEngine {
    vault: SqliteVault,
    episodic: FileEpisodicStore,
    gateway: Arc<dyn LlmGateway>,
    verify_gateway: Option<Arc<dyn LlmGateway>>,
    tools: Arc<ToolRegistry>,
    config: SparkConfig,
    synapse: Option<Arc<SynapseBus>>,
}

pub fn new_with_verify(
    gateway: Arc<dyn LlmGateway>,
    verify_gateway: Arc<dyn LlmGateway>,
    // ...
) -> Self {
    Self { verify_gateway: Some(verify_gateway), ... }
}
```

### Hypothesis / verdict schemas

```35:51:gzmo-core/src/spark.rs
struct SparkHypothesis {
    internal_analysis: String,
    anchor_label: String,
    recent_label: String,
    connection: String,
    what_to_remember: Vec<String>,
}

struct SparkVerdict {
    supported: bool,
    confidence: f64,
    evidence_anchor: String,
    evidence_recent: String,
}
```

### Dice scheduling

```7:31:gzmo-core/src/spark_schedule.rs
pub fn advance_seed(seed: u64) -> u64 {
    seed.wrapping_mul(1_103_515_245).wrapping_add(12_345)
}

pub fn next_spark_after(now: DateTime<Utc>, config: &SparkConfig, seed: u64) -> (DateTime<Utc>, u64, u32, u32) {
    let (roll, minutes) = roll_interval_minutes(seed, config.dice_min_minutes, config.dice_max_minutes);
    let next = now + Duration::minutes(minutes as i64);
    (next, next_seed, roll, minutes)
}
```

### Daemon loop (cron vs dice)

```379:459:gzmo-cli/src/daemon_cmd.rs
    let spark_handle = tokio::spawn(async move {
        let cron_slot = match spark_config.schedule_mode {
            SparkScheduleMode::Cron => spark_cron_slot_due(&now, &spark_config.cron_hours, ...),
            SparkScheduleMode::Dice => {
                if next_dice_run.is_some_and(|t| now >= t) { Some((now.hour(), now.minute())) }
                else { None }
            }
        };
        match spark_engine_clone.run(today).await {
            Ok(report) => append_spark_to_dreams(&dreams_path_spark, &report.section).await?,
        }
    });
```

Multi-hour cron uses `spark_cron_slot_due` from `daemon.rs` to pick earliest missed slot.

---

## 3. Interfaces

| Interface | Config |
|-----------|--------|
| Enable | `[spark] enabled` |
| Schedule | `schedule_mode = "cron"` \| `"dice"` |
| Cron | `cron_hours = [3, 22]`, `cron_minute = 30` |
| Dice | `dice_min_minutes`, `dice_max_minutes`, `dice_seed` |
| Gateways | `TaskKind::SparkHypothesis`, `TaskKind::SparkVerify` |
| Output | Appends `## Spark — {date}` to DREAMS.md |
| Anchor age | `max_session_anchor_age_days` (vault cleanup) |

---

## 4. THINKING nodes

> **THINKING — spark.rs:L3 promotion only**
> - *Reviewed:* Promotes HYPOTHESIZED_LINK relations, not raw L2 truths.
> - *Insight:* Serendipity without polluting honeypot with unverified facts.
> - *Risk / limitation:* Unsupported / below-threshold links abstain (no quarantine promotion). A lying `supported=true` can still write L3.
> - *Enhancement:* Decay L3 links without re-verification after N days. [GZMO-next]

> **THINKING — spark_schedule.rs:LCG dice**
> - *Reviewed:* Deterministic seed chain from chaos seed or `dice_seed`.
> - *Insight:* Reproducible schedule for debugging; chaos-free by design.
> - *Risk / limitation:* Predictable intervals if seed leaked.
> - *Enhancement:* Optional entropy injection from vault hash. [GZMO-next]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| `cognition-smoke.sh` | Lab spark path with `--spark-run` |
| CT101 | Cron 03:30 UTC + cloud GLM hypothesis/verify |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | L3 link TTL and re-verify | [GZMO-next] |
| 2 | Spark selection bias telemetry | [CT101-safe] |
| 3 | Skip spark when vault stale-anchor pool empty | [CT101-safe] |
| 4 | Integrate chaos seed only as dice init (documented) | [CT101-safe] |
