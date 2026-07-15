# Subsystem — Health Probes

**Source:** `gzmo-core/src/health.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Startup and CLI health checks for every external dependency: LLM (`/models`), embeddings, Qdrant collection, rerank, librarian, Redis PING, Neo4j Bolt TCP, MCP `read_graph`, and optional sovereign engine. Emits Synapse `HealthTick` / `HealthFail` events; strict mode aborts daemon startup.

---

## 2. How it works

### Probe result type

```19:42:gzmo-core/src/health.rs
pub struct ProbeResult {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}
```

### LLM liveness

```45:66:gzmo-core/src/health.rs
pub async fn probe_llm_models(profile: &EngineProfileConfig) -> ProbeResult {
    let url = format!("{base}/models");
    let client = Client::builder().timeout(Duration::from_secs(8)).build()?;
    // GET with optional bearer auth
}
```

### Redis, Qdrant, Neo4j, MCP

- `probe_redis` — PING with 3s connect timeout
- `probe_qdrant` — GET `/collections/{name}` → points_count
- `probe_neo4j_bolt` — TCP connect 3s
- `probe_mcp_memory` — dispatch `mcp__memory__read_graph`

### Startup orchestration

```240:350:gzmo-core/src/health.rs
pub async fn run_startup_probes(
    config: &GzmoConfig,
    tools: Option<&ToolRegistry>,
    strict: bool,
    synapse: Option<&SynapseBus>,
) -> Result<Vec<ProbeResult>> {
    // Synapse HealthTick
    let prime = config.engine.active_engine_for_mode(EngineMode::Local);
    results.push(probe_llm_models(&prime).await);
    results.push(probe_embeddings(...).await);
    results.push(probe_qdrant(...).await);
    // ... rerank, librarian, redis ...
    // Neo4j + MCP if memory server configured
    if strict && !failures.is_empty() {
        // Synapse HealthFail, bail!
    }
}
```

### Daemon wiring

```201:213:gzmo-cli/src/daemon_cmd.rs
    } else if let Err(e) = health::run_startup_probes(
        config,
        Some(dream_tools.as_ref()),
        config.health.strict_startup,
        Some(&synapse),
    ).await {
        if config.health.strict_startup {
            return Err(e);
        }
    }
```

Lab path uses `ops-smoke.sh` instead when `assembly.ops_health` is lab backend.

---

## 3. Interfaces

| Interface | Value |
|-----------|-------|
| CLI | `gzmo health` → `format_report` |
| Config | `[health] strict_startup` |
| Prime probe | `[engine.local]` URL (not cloud active_mode) |
| Synapse events | `HealthTick`, `HealthFail` |
| Sovereign | Optional; failure is expected until GGUF exists |

---

## 4. THINKING nodes

> **THINKING — health.rs:prime vs active_engine**
> - *Reviewed:* Startup probes Prime/local explicitly, not cloud GLM.
> - *Insight:* CT101 `active_mode=cloud` still validates workstation fallback path at boot.
> - *Risk / limitation:* Cloud-only outage undetected if Prime also down but daemon runs.
> - *Enhancement:* Also probe `[engine.cloud]/models` when cloud_first_background. [CT101-safe]

> **THINKING — health.rs:strict_startup**
> - *Reviewed:* Non-sovereign failures bail when strict; sovereign always soft-fail.
> - *Insight:* Allows daemon to start with optional subsystems missing.
> - *Risk / limitation:* Silent degradation if operator disables strict.
> - *Enhancement:* Document CT101 recommended `strict_startup = true` for Redis/Qdrant. [CT101-safe]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| `ops-smoke.sh` | Lab assembly replaces inline probes |
| Observatory | Polls health indirectly via telemetry files |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Cloud engine probe when active_mode=cloud | [CT101-safe] |
| 2 | Periodic re-probe in heartbeat (not just startup) | [CT101-safe] |
| 3 | JSON health endpoint for Observatory | [GZMO-next] |
| 4 | Embed/rerank latency thresholds | [CT101-safe] |
