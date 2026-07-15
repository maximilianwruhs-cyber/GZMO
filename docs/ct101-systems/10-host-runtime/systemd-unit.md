# Subsystem — systemd Unit

**Sources:** `scripts/systemd/gzmo-daemon.service`, `gzmo-cli/src/daemon_cmd.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Keeps the GZMO background daemon (`gzmo daemon`) running under systemd with automatic restart. The CLI entry wires heartbeat, cognition engines, orchestrator, watchers, and chaos into a single long-lived async runtime supervised by the unit.

---

## 2. How it works

### systemd unit

```1:16:scripts/systemd/gzmo-daemon.service
[Unit]
Description=GZMO background daemon (vault, dreams, watchers)
After=network-online.target gzmo-prime.service
Wants=network-online.target

[Service]
Type=simple
User=%i
WorkingDirectory=%h/Projects/_foundation-audit/survey_GZMO
ExecStart=%h/Projects/_foundation-audit/survey_GZMO/target/release/gzmo daemon
Restart=on-failure
RestartSec=30
# VM200 embed/rerank must be up separately (deploy-retrieval-layer on VM200).

[Install]
WantedBy=default.target
```

On CT101 production, paths resolve to `/opt/gzmo/survey_GZMO` and user `maximilian` (see [CT101_INFRASTRUCTURE_REPORT.md](../../CT101_INFRASTRUCTURE_REPORT.md)).

### CLI daemon entry

`daemon_cmd::run` bootstraps all subsystems and blocks on `tokio::select!` until a task exits:

```52:77:gzmo-cli/src/daemon_cmd.rs
pub async fn run(config: &GzmoConfig, identity: IdentityEngine) -> Result<()> {
    let soul = identity.snapshot().await;

    info!("╔══════════════════════════════════════════════╗");
    info!("║            GZMO — Daemon Mode                ║");
    info!("║       100% Local · Air-Gapped · Rust         ║");
    info!("╚══════════════════════════════════════════════╝");
    info!(persona = %soul.persona_name, "Identity loaded");

    // Assembly backends — lab recipes only activate under GZMO_INSTANCE=next
    // (AssemblyConfig::effective forces Inline otherwise; CT101-safe).
    let asm = &config.assembly;
    let distill_backend = asm.effective(asm.distill);
    let dream_backend = asm.effective(asm.dream);
    let spark_backend = asm.effective(asm.spark);
    let ops_backend = asm.effective(asm.ops_health);
    let handoff_backend = asm.effective(asm.config_handoff);
```

Gateway router and per-task LLM bindings are created at startup:

```102:110:gzmo-cli/src/daemon_cmd.rs
    // Gateway + Tools for dream cycle — use Obolus GatewayRouter
    let router = GatewayRouter::new(config);
    let dream_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DreamExtract));
    let dream_verify_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DreamVerify));
    let ingest_verify_gateway: Arc<dyn LlmGateway> =
        Arc::clone(router.gateway(TaskKind::IngestVerify));
    let spark_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::SparkHypothesis));
    let spark_verify_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::SparkVerify));
    let ingest_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::IngestExtract));
```

The main loop pins all background tasks:

```765:777:gzmo-cli/src/daemon_cmd.rs
    tokio::select! {
        _ = heartbeat_handle => error!("Heartbeat exited"),
        _ = dream_handle => error!("Dream cycle exited"),
        _ = spark_handle => error!("Spark cycle exited"),
        _ = qdrant_handle => error!("Qdrant sync loop exited"),
        _ = distill_handle => error!("Session distill loop exited"),
        _ = distill_worker_handle => error!("Distill archive worker exited"),
        _ = synapse_handle => error!("Synapse pull loop exited"),
        _ = kg_handle => error!("KG reconcile loop exited"),
        _ = wiki_sync_handle => error!("Wiki sync loop exited"),
        _ = wiki_lint_handle => error!("Wiki lint loop exited"),
        _ = handoff_handle => error!("Config handoff loop exited"),
    }
```

---

## 3. Interfaces

| Interface | CT101 production | Repo template |
|-----------|------------------|---------------|
| Unit name | `gzmo-daemon.service` | same |
| Binary | `/opt/gzmo/survey_GZMO/target/release/gzmo daemon` | `%h/.../target/release/gzmo daemon` |
| Config | `/opt/gzmo/gzmo.toml` | CWD-relative `gzmo.toml` |
| CLI subcommand | `gzmo daemon` | same |
| Depends | `network-online.target` | optional `gzmo-prime.service` |

---

## 4. THINKING nodes

> **THINKING — gzmo-daemon.service:paths**
> - *Reviewed:* `WorkingDirectory` and `ExecStart` point at workstation audit path in repo template.
> - *Insight:* CT101 uses a different filesystem layout under `/opt/gzmo/`; unit must be templated per host.
> - *Risk / limitation:* Copying repo unit verbatim to CT101 would fail to start.
> - *Enhancement:* Ship `gzmo-daemon@.service` with `%i` user and documented `/opt/gzmo` override. [CT101-safe]

> **THINKING — daemon_cmd.rs:assembly guard**
> - *Reviewed:* `AssemblyConfig::effective` forces Inline on CT101 unless `GZMO_INSTANCE=next`.
> - *Insight:* Lab shell recipes (session-to-dream.sh, cognition-smoke.sh) never run on frozen CT101.
> - *Risk / limitation:* Operator confusion if env var leaked onto CT101.
> - *Enhancement:* Log explicit "CT101 inline mode" banner at startup. [CT101-safe]

> **THINKING — daemon_cmd.rs:select loop**
> - *Reviewed:* Any spawned task exit logs error and ends daemon.
> - *Insight:* Single point of failure — one panicked cron loop kills the whole process.
> - *Risk / limitation:* systemd restarts after 30s; missed cron windows during outage.
> - *Enhancement:* Isolate cognition loops in separate supervised tasks or restart individual loops. [GZMO-next]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| `GZMO_INSTANCE=next` | Enables lab assembly backends in daemon_cmd (handoff, ops-smoke) |
| CT101 live | Inline Rust engines only; cloud GLM 5.2 via `[engine.cloud]` |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Production unit file variant for `/opt/gzmo` | [CT101-safe] |
| 2 | `MemoryMax=4G` cgroup limit in unit | [CT101-safe] |
| 3 | `WatchdogSec` heartbeat from daemon | [GZMO-next] |
| 4 | Split cognition into separate systemd services | [GZMO-next] |
