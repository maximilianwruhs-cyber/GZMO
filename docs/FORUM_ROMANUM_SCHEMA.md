# Forum Romanum MVP — Synapse Schema

Canonical contract for multi-agent coordination on the append-only bus.
See also [SYNAPSE_EVENT_OWNERSHIP.md](./SYNAPSE_EVENT_OWNERSHIP.md).

## Envelope (all MVP events)

| Field | Required | Notes |
|-------|----------|-------|
| `id` | yes | UUID v4 |
| `event_type` | yes | See types below |
| `source` | yes | `pi_agent` for agent traffic; `gzmo_daemon` for `spawn.recommended` |
| `timestamp` | yes | ISO 8601 UTC |
| `correlation_id` | recommended | Conversation / debate scope (often `session_id`) |
| `reply_to` | optional | Parent event `id` for threading |
| `data` | optional | Type-specific payload |

Legacy events without `correlation_id` / `reply_to` remain valid.

## Event types

| JSON `event_type` | Rust variant | Writer |
|-------------------|--------------|--------|
| `agent.spawned` | `AgentSpawned` | Pi (phase 2: Kurator approve) |
| `agent.message` | `AgentMessage` | Pi / pi-crew |
| `agent.result` | `AgentResult` | Pi |
| `agent.error` | `AgentError` | Pi |
| `proposal.created` | `ProposalCreated` | Pi (Prometheus role) |
| `proposal.reviewed` | `ProposalReviewed` | Pi (Epimetheus role) |
| `spawn.recommended` | `SpawnRecommended` | GZMO Kurator monitor |

## `agent.message` data payload

```json
{
  "agent_id": "prometheus",
  "role": "proposer",
  "mode": "debate",
  "payload": {
    "text": "...",
    "artifacts": []
  }
}
```

**Modes** (dialectics as conventions, not a runtime): `debate`, `synthesize`, `explore`.

## `proposal.created` / `proposal.reviewed`

```json
{
  "agent_id": "prometheus",
  "proposal_id": "uuid",
  "title": "...",
  "body": "...",
  "status": "draft"
}
```

```json
{
  "agent_id": "epimetheus",
  "proposal_id": "uuid",
  "verdict": "accept",
  "comments": "..."
}
```

## Example threaded chain

```json
{"id":"a1","event_type":"session_start","source":"pi_agent","timestamp":"...","correlation_id":"sess-uuid","data":{"session_id":"sess-uuid"}}
{"id":"a2","event_type":"agent.message","source":"pi_agent","timestamp":"...","correlation_id":"sess-uuid","reply_to":"a1","data":{"agent_id":"prometheus","role":"proposer","mode":"debate","payload":{"text":"..."}}}
{"id":"a3","event_type":"agent.message","source":"pi_agent","timestamp":"...","correlation_id":"sess-uuid","reply_to":"a2","data":{"agent_id":"epimetheus","role":"critic","mode":"debate","payload":{"text":"..."}}}
```

## Implementation status

- Rust: enum variants + envelope fields in `gzmo-core/src/synapse.rs`
- Rust: Forum Romanum emitters in `gzmo-core/src/synapse_writer.rs` (`emit_agent_*`, `emit_proposal_*`)
- Pi: `scripts/pi/forum-romanum-bridge.reference.ts` maps pi-crew `crewHooks` -> bus (live: `~/.pi/agent/extensions/forum-romanum-bridge.ts`)
- Fixture: `scripts/pi/emit_forum_romanum_fixture.py` for CI without pi-crew run
- Kurator: `spawn.recommended` emitted by `kurator_monitor.rs`; `gzmo kurator approve` spawns sub-agent + writes `agent.spawned` / `agent.result` (phase 2)
