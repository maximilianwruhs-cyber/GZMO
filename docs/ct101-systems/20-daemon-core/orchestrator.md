# Subsystem — Orchestrator

**Source:** `gzmo-core/src/orchestrator.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Runs configured cron jobs as headless agent conversations. Supports **simple mode** (single prompt) and **pipeline mode** (multi-step DAG with parallel waves, retries, vault persistence, and Synapse observability).

---

## 2. How it works

### Context shared by all jobs

```74:95:gzmo-core/src/orchestrator.rs
pub struct OrchestratorContext {
    pub gateway: Arc<dyn LlmGateway>,
    pub tools: Arc<ToolRegistry>,
    pub system_prompt: String,
    pub vault: Option<Arc<SqliteVault>>,
    pub episodic: Option<Arc<FileEpisodicStore>>,
    pub chaos_feedback_tx: Option<tokio::sync::mpsc::Sender<gzmo_chaos::feedback::ChaosEvent>>,
    pub ingest_engine: Option<Arc<crate::ingest::IngestEngine>>,
    pub synapse: Option<Arc<SynapseBus>>,
    pub scratch: Arc<ScratchService>,
    pub memory_search_scope: Arc<std::sync::Mutex<ScratchScope>>,
    pub context: ContextConfig,
}
```

### Scheduler boot

```188:289:gzmo-core/src/orchestrator.rs
pub async fn start_orchestrator(
    jobs: HashMap<String, JobConfig>,
    ctx: Arc<OrchestratorContext>,
) -> Result<JobScheduler> {
    let sched = JobScheduler::new().await?;
    // ... filter disabled jobs ...
    for (name, job_config) in active_jobs {
        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            Box::pin(async move {
                // Synapse: DaemonTick
                let outcome = execute_job(&ctx, &job_name, &job_cfg).await;
                // Synapse: DaemonJobComplete / DaemonJobFail
            })
        })?;
        sched.add(job).await?;
    }
    sched.start().await?;
    Ok(sched)
}
```

### Wave resolution (topological sort)

```106:179:gzmo-core/src/orchestrator.rs
fn resolve_waves(steps: &[JobStep]) -> Result<Vec<Vec<usize>>> {
    // Build in-degree map, adjacency list
    // Collect steps with in_degree == 0 → wave
    // Error on circular dependencies
}
```

### Pipeline execution

Parallel steps within a wave use `tokio::spawn`; sequential waves respect `depends_on`:

```421:442:gzmo-core/src/orchestrator.rs
    for (wave_idx, wave) in waves.iter().enumerate() {
        if pipeline_failed {
            // mark remaining as Skipped
            continue;
        }
        info!(job = %job_name, wave = wave_idx + 1, steps = wave.len(), "Pipeline: executing wave");
        if wave.len() == 1 {
            // sequential single step
        } else {
            // parallel execution within wave
        }
    }
```

### Daemon integration

Spark and auto_dream are **removed** from orchestrator jobs — handled by dedicated engines:

```297:303:gzmo-cli/src/daemon_cmd.rs
    let mut orch_jobs = config.orchestration.jobs.clone();
    orch_jobs.remove("spark");
    orch_jobs.remove("auto_dream");
    let _scheduler = match gzmo_core::orchestrator::start_orchestrator(orch_jobs, Arc::clone(&orch_ctx)).await {
```

---

## 3. Interfaces

| Interface | Location |
|-----------|----------|
| Job definitions | `[orchestration.jobs.<name>]` in `gzmo.toml` |
| Cron expressions | `cron = "0 5 * * *"` per job |
| Pipeline steps | `[[orchestration.jobs.<name>.steps]]` with `depends_on` |
| Gateway task | `TaskKind::Daemon` via `OrchestratorContext` |
| Public headless API | `execute_headless(ctx, job_name, prompt)` — used by watchers |

---

## 4. THINKING nodes

> **THINKING — orchestrator.rs:resolve_waves**
> - *Reviewed:* Kahn-style topological sort with cycle detection.
> - *Insight:* Diamond DAGs (a→b,c→d) execute b+c in parallel — good for IO-bound tool steps.
> - *Risk / limitation:* Parallel steps share no scratch scope isolation beyond step name.
> - *Enhancement:* Per-step scratch namespaces for parallel safety. [GZMO-next]

> **THINKING — orchestrator.rs:scan_skills_metadata**
> - *Reviewed:* Dynamically lists `./skills/*.sh` for Host-Parasite prompts.
> - *Insight:* Orchestrator jobs can invoke discovery scripts without config changes.
> - *Risk / limitation:* Reads filesystem synchronously on every step.
> - *Enhancement:* Cache skills listing with mtime invalidation. [CT101-safe]

> **THINKING — daemon_cmd.rs:job removal**
> - *Reviewed:* `spark` and `auto_dream` stripped from orchestrator map.
> - *Insight:* Prevents double-firing now that SparkEngine/DreamEngine own those schedules.
> - *Risk / limitation:* Stale entries in production `gzmo.toml` are silently ignored.
> - *Enhancement:* Startup warning if removed job names still present in config. [CT101-safe]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| Discovery automation | Jobs under `[orchestration.jobs]` drive auto-socratic cycles |
| GZMO-next scheduler | Separate `gzmo-scheduler` crate for next-stack cron |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Warn on deprecated job names (spark, auto_dream) | [CT101-safe] |
| 2 | Job outcome dashboard via Synapse aggregates | [CT101-safe] |
| 3 | Pipeline step timeouts per step | [GZMO-next] |
| 4 | Job dependency across daemon restarts (checkpoint) | [GZMO-next] |
