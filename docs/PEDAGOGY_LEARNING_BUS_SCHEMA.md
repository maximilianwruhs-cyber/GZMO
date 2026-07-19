> **Recovered 2026-07-19** from `origin/feat/context-compress-headroom`. See [LOST_KNOWLEDGE_INVENTORY.md](./LOST_KNOWLEDGE_INVENTORY.md).

# Pedagogy Learning — Synapse Bus Schema

Canonical contract for falsifiable learning evidence after pedagogy tension oscillation (F5).

See also: [FORUM_ROMANUM_SCHEMA.md](./FORUM_ROMANUM_SCHEMA.md), [SYNAPSE_EVENT_OWNERSHIP.md](./SYNAPSE_EVENT_OWNERSHIP.md).

## Principles

1. **Append-only:** GZMO daemon writes events; it does not consume bus content for PulseLoop state.
2. **`learning_verified`** is computed by external operator tools (`verify-learning-after-oscillation.sh`), not written by the daemon on `oscillation_complete`.
3. **`oscillation_id`** threads the full chain: oscillation → discovery spawn → Pi `mentor_teach` / `proposal.*`.

## Envelope

All `pedagogy.oscillation_*` events use:

| Field | Required | Notes |
|-------|----------|-------|
| `correlation_id` | yes (v2+) | UUID v4 = `oscillation_id` for the cycle |
| `event_type` | yes | `pedagogy.oscillation_start` \| `_step` \| `_complete` |
| `source` | yes | `gzmo_daemon` |
| `data.oscillation_id` | yes (v2+) | Same UUID as envelope (grep-friendly) |

## Event payloads

### `pedagogy.oscillation_start`

```json
{
  "oscillation_id": "uuid",
  "step": 1,
  "target": 0.9,
  "label": "High tension — confirmation machine",
  "duration_secs": 60,
  "is_low_phase": false,
  "chaos_val": 0.94,
  "chaos_val_raw": 0.94,
  "chaos_val_baseline": 0.94,
  "knowledge_state_before": {
    "known_nodes": [],
    "open_gaps": [],
    "source": "empty"
  }
}
```

### `pedagogy.oscillation_step`

Same fields as today plus `oscillation_id`. `is_low_phase: true` triggers optional discovery spawn.

### `pedagogy.oscillation_complete`

```json
{
  "oscillation_id": "uuid",
  "step": 0,
  "target": 0.0,
  "label": "cycle complete",
  "duration_secs": 0,
  "is_low_phase": false,
  "chaos_val": 0.55,
  "chaos_val_raw": 0.55,
  "chaos_val_baseline": 0.94,
  "knowledge_state_after": {},
  "knowledge_delta": {
    "added": [],
    "changed": [],
    "removed": []
  },
  "spawned_tasks": [
    {"session_id": "auto-pedagogy_oscillation-...", "trigger": "pedagogy_oscillation"}
  ]
}
```

### `pedagogy.learning_certified` (Layer 3)

Written by operator CLI `gzmo pedagogy certify`.

```json
{
  "oscillation_id": "uuid",
  "learning_verified": true,
  "certified_by": "operator",
  "layer": 3
}
```

## Pi events (threading)

When discovery is spawned from oscillation, Pi should set:

- `GZMO_CORRELATION_ID=<oscillation_id>` → all `mentor_teach`, `quest_complete`, `session_start` use this `correlation_id`.

Optional on `mentor_teach.data`:

- `confidence_score`: 0.0–1.0 (Layer 2 predicate)
- `novel_application`: bool (Layer 2 / R8)

## Six-predicate acceptance (Layer 1)

External tool checks (same `correlation_id` / `oscillation_id`):

1. `oscillation_complete` exists AND `knowledge_delta` non-empty
2. Downstream `mentor_teach` or `proposal.created` references delta concepts with novel application
3. At least one step with `is_low_phase: true`
4. `|chaos_val_complete - chaos_val_baseline| > 0.05`
5. Mentor `confidence_score >= 0.6` on correlated `mentor_teach`
6. Layer 2 manual or structured Mentor confirmation

## Handoff v2 → `knowledge_state_before`

When `GZMO_HANDOFF_PATH` points to a v2 handoff JSON at cycle start, daemon maps:

- `concepts_established` → `known_nodes`
- `gaps_identified` → `open_gaps`
- `source`: `"socratic_handoff"`
