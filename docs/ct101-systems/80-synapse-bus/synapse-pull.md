# Synapse Pull — Pi Event Reader

**System:** [80-synapse-bus](./SYSTEM.md)  
**Source:** `gzmo-core/src/synapse_reader.rs`

---

## Capability

`synapse_reader` is the **only sanctioned read path** for Synapse in core: tail-new Pi events (`EventSource::PiAgent`) from byte offset, summarize quest/session activity, append pull summary to **episodic**, and enqueue **distill jobs** on `session_end` events. State persisted in `data/synapse-reader.state.json` — does not mutate the bus file.

---

## How it works

### Incremental read

```45:91:gzmo-core/src/synapse_reader.rs
pub fn read_new_pi_events(bus_path: &Path, state_path: &Path, max_events: usize) -> Result<(Vec<SynapseEvent>, SynapseReaderState)> {
    // seek to state.byte_offset
    // parse lines; filter source == PiAgent
    // advance offset, save state
}
```

### Session end → distill

```128:161:gzmo-core/src/synapse_reader.rs
pub async fn enqueue_session_end_distills(scratch: &ScratchService, events: &[SynapseEvent], fallback_transcript: &str) -> Result<usize> {
    let handoffs = extract_session_end_handoffs(events);
    for h in handoffs {
        scratch.enqueue_distill(DistillJob { session_id, transcript, source: DistillSource::MainArchive }).await?;
    }
}
```

### Episodic integration

```217:253:gzmo-core/src/synapse_reader.rs
pub async fn pull_and_log_episodic(/* bus, state, episodic, scratch */) -> Result<PiEventSummary> {
    let (events, state) = read_new_pi_events(/* ... */)?;
    // enqueue distills if scratch provided
    episodic.append(&EpisodicEntry { content: summary, /* ... */ }).await?;
}
```

`extract_session_end_handoffs` reads `session_id`, `reason`, `sessionFile` from event JSON payload.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Bus path | `data/Synapse/events.jsonl` (~488k events) |
| State | `data/synapse-reader.state.json` — `byte_offset`, `events_processed` |
| Trigger | Daemon/CLI synapse pull command or scheduled job |
| Outputs | Episodic markdown; Redis distill queue via `ScratchService` |
| Filter | **Pi agent only** — ignores gzmo_daemon/cli events in pull |

---

## THINKING nodes

> **THINKING — synapse_reader.rs:read_new_pi_events**
> - *Reviewed:* Byte offset not line-based; reset offset if file truncated.
> - *Insight:* Efficient tail for large 488k JSONL without re-scanning.
> - *Risk / limitation:* Invalid JSON lines skipped silently — offset still advances.
> - *Enhancement:* Count parse failures in PiEventSummary [CT101-safe].

> **THINKING — synapse_reader.rs:session_end handoff**
> - *Reviewed:* Reads transcript from session_file path when present.
> - *Insight:* Closes Pi → GZMO memory loop without Pi writing vault directly.
> - *Risk / limitation:* Missing file falls back to stub transcript — weak distill input.
> - *Enhancement:* Synapse payload require session_file on CT101 [CT101-safe].

> **THINKING — synapse_reader vs Prime Directive**
> - *Reviewed:* Reader is separate module; not a bus subscriber inside writer.
> - *Insight:* Pull is batch ETL, not reactive chemistry — preserves firewall intent.
> - *Risk / limitation:* Multiple pullers could duplicate distills if state files diverge.
> - *Enhancement:* Single-writer lock on state file [CT101-safe].

---

## Advancement

- **CT101:** Schedule pull after Pi sessions; monitor `distill_enqueued` in logs.
- **GZMO-next:** Idempotent distill by session_id dedup in queue consumer.

---

## Enhancement backlog

1. **[CT101-safe]** fs2 lock on synapse-reader.state.json during pull.
2. **[CT101-safe]** Require non-empty transcript for session_end distill enqueue.
3. **[CT101-safe]** Report parse error count in pull summary.
4. **[GZMO-next]** Pull gzmo_daemon dream_complete events for cross-stack dashboard (read-only).
5. **[GZMO-next]** session_id dedup in distill worker before LLM call.
