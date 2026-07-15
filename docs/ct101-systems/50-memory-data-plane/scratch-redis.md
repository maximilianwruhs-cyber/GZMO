# Scratch & Redis — Ephemeral Recall & Distill Queue

**System:** [50-memory-data-plane](./SYSTEM.md)  
**Source:** `gzmo-core/src/memory/scratch.rs`

---

## Capability

`ScratchService` stores per-session **recall snippets** in Redis (or in-memory fallback) under keys `gzmo:scratch:{main|sub|orch}:...`. It formats `[RECALL]` blocks for LLM injection, enforces token budgets, and enqueues **distill jobs** to Redis BRPOP queue (with filesystem fallback dir). Embedding cache shares the same Redis sidecar on CT101.

---

## How it works

### Scope keys

```22:38:gzmo-core/src/memory/scratch.rs
pub enum ScratchScope {
    Main { session_id: String },
    Sub { session_id: String, task_id: String },
    Orch { job: String, step: String },
}
impl ScratchScope {
    pub fn redis_key(&self) -> String { /* gzmo:scratch:... */ }
```

### Resilient Redis backend

```105:138:gzmo-core/src/memory/scratch.rs
    async fn conn(&self) -> Result<redis::aio::ConnectionManager> {
        // 3s timeout, 15s reconnect backoff
        // on failure: bail → caller uses fallback HashMap buffer
    }
```

Writes buffer to `fallback` when Redis down; reads prefer Redis then fallback.

### Recall injection

```340:349:gzmo-core/src/memory/scratch.rs
    pub async fn format_for_inject(&self, scope: &ScratchScope) -> Result<Option<String>> {
        let mut lines = vec!["[RECALL]".to_string()];
        // snippets with score, optional evidence_text, token budget trim
```

Distill jobs pushed to `[redis] distill_queue`; fallback JSON files in `distill_fallback_dir` when queue unavailable.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Config `[redis]` | `enabled`, `url`, `distill_queue`, `distill_fallback_dir` |
| Config `[context_memory]` | `scratch_max_tokens` (default 2000) |
| Sidecar | Redis on CT101 `:6379` |
| Writers | `PlatformMemory::memory_search`, agent recall tools |
| Readers | Agent loop inject, `gzmo_memory_recall_pull` MCP tool |
| Synapse | `enqueue_session_end_distills` → distill queue on Pi session_end |

---

## THINKING nodes

> **THINKING — scratch.rs:RedisBackend fallback**
> - *Reviewed:* Lazy reconnect vs permanent memory pin at startup failure.
> - *Insight:* Single-turn scratch survives Redis blip — important for long daemon uptime.
> - *Risk / limitation:* Fallback is process-local — subagent on different task loses scratch.
> - *Enhancement:* Sticky session affinity or always-Redis for subagents [GZMO-next].

> **THINKING — scratch.rs:DistillJob queue**
> - *Reviewed:* Redis list + filesystem fallback for session_distill worker.
> - *Insight:* Decouples Pi synapse pull from heavy distill LLM work.
> - *Risk / limitation:* Fallback dir can fill disk if worker stuck.
> - *Enhancement:* Max depth alert on distill queue [CT101-safe].

> **THINKING — scratch + embed cache sharing Redis**
> - *Reviewed:* Separate key prefixes (`gzmo:scratch:` vs `gzmo:embed:`).
> - *Insight:* One sidecar serves two hot paths — ops simplicity on CT101.
> - *Risk / limitation:* Memory pressure on Redis evicts embed cache first (TTL) not scratch.
> - *Enhancement:* Redis memory policy monitoring in health [CT101-safe].

---

## Advancement

- **CT101:** Ensure sidecar-redis restarts don't lose distill queue — persist AOF/RDB on CT101.
- **GZMO-next:** Scratch TTL per scope to prevent unbounded key growth on forgotten sessions.

---

## Enhancement backlog

1. **[CT101-safe]** Distill queue depth + oldest job age in `gzmo health`.
2. **[CT101-safe]** Scratch key TTL (24h) for abandoned sessions.
3. **[CT101-safe]** Flush fallback buffer to Redis on reconnect.
4. **[GZMO-next]** Structured scratch schema (fact_id, evidence, source_file) for UI recall panel.
5. **[GZMO-next]** Separate Redis DB index for embed cache vs scratch vs distill.
