# Synapse Writer — Append-Only Event Bus

**System:** [80-synapse-bus](./SYSTEM.md)  
**Source:** `gzmo-core/src/synapse.rs`

---

## Capability

`SynapseBus` appends structured **`SynapseEvent`** frames to JSONL with **fs2 exclusive lock** on a companion `.lock` file. Fire-and-forget semantics — failures log warnings, never block cognition. Thread-local **`EventSource`** tags events by writer context. Live volume: **488,944** lines on CT101 (2026-07-14).

---

## How it works

### Event model

```129:178:gzmo-core/src/synapse.rs
pub struct SynapseEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub source: EventSource,
    pub timestamp: DateTime<Utc>,
    pub data: Option<serde_json::Value>,
}

impl SynapseEvent {
    pub fn with_data(event_type: EventType, source: EventSource, data: serde_json::Value) -> Self { /* ... */ }
    pub fn to_json_line(&self) -> String { /* serde_json one line */ }
```

`EventType` covers daemon lifecycle, health, quest/session, memory ops (dream/spark/ingest/distill/wiki), senses (`SenseChaosRho`), and Pi interactions.

### Append with lock

```220:302:gzmo-core/src/synapse.rs
    pub fn append(&self, event: &SynapseEvent) {
        let line = event.to_json_line();
        // open events.jsonl.lock → lock_exclusive → append line → drop
    }

    pub fn append_batch(&self, events: &[SynapseEvent]) { /* single lock, multiple lines */ }
```

### Thread-local source

```45:52:gzmo-core/src/synapse.rs
pub fn set_event_source(source: EventSource) {
    SYNAPSE_SOURCE.with(|s| s.set(Some(source)));
}
```

### Prime Directive (module docs)

```6:11:gzmo-core/src/synapse.rs
//! **The Firewall:** GZMO does not run an active "chemistry consumer"...
//! **The Prime Directive:** This module writes events only. It never reads,
//! never consumes, never triggers state changes based on bus content.
```

---

## Interfaces

| Kind | Detail |
|------|--------|
| Default path | `data/Synapse/events.jsonl` |
| Lock file | `data/Synapse/events.jsonl.lock` |
| Sources | `gzmo_daemon`, `gzmo_cli`, `pi_agent` |
| API | `SynapseBus::append`, `append_batch`; `Arc<SynapseBus>` shared in daemon |
| Not allowed | Automated subscribe/consume loops in core daemon |

---

## THINKING nodes

> **THINKING — synapse.rs:append_line**
> - *Reviewed:* create_dir_all, lock_exclusive, best-effort write without lock on failure.
> - *Insight:* Cross-process safety for daemon + CLI + Pi on same CT101 data dir.
> - *Risk / limitation:* Best-effort unlocked write can interleave JSONL on lock failure.
> - *Enhancement:* Retry once on lock failure; metric locked vs unlocked writes [CT101-safe].

> **THINKING — synapse.rs:EventType taxonomy**
> - *Reviewed:* Memory ops + senses separated; SenseChaosRho renamed in serde.
> - *Insight:* Clear domain.action naming for Observatory filters.
> - *Risk / limitation:* No schema version — evolving payloads may break downstream parsers.
> - *Enhancement:* Optional `schema_version` field on events [GZMO-next].

> **THINKING — synapse.rs:fire-and-forget**
> - *Reviewed:* Errors only tracing::warn; never propagate to caller.
> - *Insight:* Observability must not crash dream/ingest critical paths.
> - *Risk / limitation:* Silent data loss if disk full — only warnings.
> - *Enhancement:* Disk space preflight in health tick [CT101-safe].

---

## Advancement

- **CT101:** Rotate JSONL when >500k lines; archive to compressed monthly files.
- **GZMO-next:** Optional read API crate separate from writer (Observatory SDK).

---

## Enhancement backlog

1. **[CT101-safe]** Log rotation script + cron on CT101.
2. **[CT101-safe]** Disk free space alert tied to Synapse path filesystem.
3. **[CT101-safe]** Count events by type in daily health summary.
4. **[GZMO-next]** Event schema registry document auto-generated from EventType enum.
5. **[GZMO-next]** Read-only tail HTTP endpoint for Observatory (no daemon consume).
