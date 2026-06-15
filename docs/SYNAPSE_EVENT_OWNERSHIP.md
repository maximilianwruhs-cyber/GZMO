# Synapse Event Ownership Matrix

**Purpose:** Define which component owns each `event_type` on the append-only bus
(`data/Synapse/events.jsonl`). Prevents ambiguous duplicate rows when Pi and GZMO
both observe the same operation.

## Rules

1. **Pi-owned** events carry `source: pi_agent` and `data.session_id` (UUID set at
   `session_start`).
2. **GZMO-owned** events carry `source: gzmo_daemon` or `gzmo_cli`.
3. When Pi echoes a GZMO engine completion (tool call observed), set
   `data.emitted_by: "pi_tool_echo"` so tailers can dedupe against the canonical
   Rust engine event.
4. GZMO core never consumes bus content for state transitions (firewall). Readers
   live in `synapse_reader.rs`, `kurator_monitor.rs`, and Pi extensions only.

## Ownership table

| event_type | Owner | Writer | Notes |
|------------|-------|--------|-------|
| `session_start` | Pi | synapse-notifier | Sets `session_id` |
| `session_end` | Pi | synapse-notifier | `targetSessionFile` for distill |
| `quest_complete` | Pi | synapse-notifier | Per-turn audit |
| `quest_fail` | Pi | synapse-notifier | Turn/tool failure |
| `mentor_teach` | Pi | synapse-notifier | Socratic exchange |
| `mentor_learn_start` / `mentor_learn_end` | Pi | synapse-notifier | Learn mode |
| `topic_shift_distill` | Pi | synapse-notifier | Mid-session distill trigger |
| `skill.invoke` / `skill.complete` / `skill.error` | Pi | synapse-notifier | Chaos skill boundary |
| `agent.*` / `proposal.*` | Pi | synapse-notifier | Forum Romanum (see FORUM_ROMANUM_SCHEMA.md) |
| `dream_complete` | GZMO | DreamEngine | Canonical engine completion |
| `dream_complete` (echo) | Pi | synapse-notifier | `emitted_by: pi_tool_echo` |
| `spark_complete` | GZMO | SparkEngine | Canonical |
| `spark_complete` (echo) | Pi | synapse-notifier | `emitted_by: pi_tool_echo` |
| `ingest_complete` | GZMO | IngestEngine | Canonical |
| `ingest_complete` (echo) | Pi | synapse-notifier | `emitted_by: pi_tool_echo` |
| `distill_complete` | GZMO | SessionDistillEngine | Canonical |
| `distill_complete` (echo) | Pi | synapse-notifier | `emitted_by: pi_tool_echo` |
| `wiki_complete` | Pi | synapse-notifier | No Rust writer today |
| `health_tick` / `health_fail` | GZMO | health.rs | Probes |
| `health_tick` (echo) | Pi | synapse-notifier | Mentor ping/status |
| `daemon_*` | GZMO | orchestrator | Scheduled jobs |
| `chaos.rho_telemetry` | GZMO | chaos_bootstrap | PulseLoop snapshot |
| `chaos.dice_loop` | GZMO | daemon_cmd | Würfel autopoietic roll |
| `chaos.feedback_drained` | GZMO | chaos_bootstrap | Skill IPC batch |
| `spawn.recommended` | GZMO | kurator_monitor | Read-only recommendation |

## session_id convention

- Generated as UUID v4 on `session_start`.
- Copied into `data.session_id` on every Pi event in that session.
- Becomes `correlation_id` on the envelope for Forum Romanum multi-agent flows
  (Stage 3).

## Backward compatibility

Events without `session_id` or `emitted_by` remain valid. Readers must not require
these fields.
