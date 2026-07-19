# Spawn Gate — Autospawn Policy

> **Recovered 2026-07-19** from `origin/feat/context-compress-headroom`. See [LOST_KNOWLEDGE_INVENTORY.md](./LOST_KNOWLEDGE_INVENTORY.md).

Central immunsystem-style gate for Kurator autospawns (not manual `gzmo kurator approve`).

## Tiers

| Kind | Trigger | Profile | Autospawn config |
|------|---------|---------|------------------|
| `discovery_fix` | Discovery FAIL/GAP report | epimetheus | `[kurator.spawn_gate] auto_spawn_discovery_fix` |
| `session_triage` | Pi metrics / dice loops | prometheus | `[kurator] auto_spawn_on_recommend` |

**Tier rule:** When `prometheus_requires_idle = true`, session_triage autospawn is blocked while a discovery_fix recommendation is pending or was spawned within `spawn_cooldown_secs`.

## Policies (`[kurator.spawn_gate]`)

| Key | Default | Effect |
|-----|---------|--------|
| `enabled` | `true` | Master switch |
| `max_autospawns_per_hour` | `3` | Global hourly budget |
| `spawn_cooldown_secs` | `600` | Min gap between any autospawns |
| `duplicate_reason_max_per_hour` | `3` | Circuit breaker per reason hash |
| `prometheus_requires_idle` | `true` | Fixer before triage |
| `auto_spawn_discovery_fix` | `true` | Discovery fixer autospawn |
| `prime_budget_enabled` | `true` | Redis hourly Prime slot budget |
| `prime_spawn_budget_per_hour` | `3` | Max autospawns consuming Prime per hour |
| `prime_budget_fail_open` | `true` | Allow spawn if Redis unreachable |
| `prime_budget_ttl_secs` | `7200` | Redis key TTL |

## Prime budget (Redis LXC101)

Before each autospawn, one slot is acquired in Redis (`gzmo:spawn:prime:hour:YYYYMMDDHH`).
Protects Prime `:8000` from concurrent sub-agent LLM load. Manual approve bypasses.

```bash
redis-cli -h 192.168.31.202 GET "gzmo:spawn:prime:hour:$(date -u +%Y%m%d%H)"
```

Denial code: `prime_budget_exhausted` or `prime_budget_unavailable` (when `fail_open=false`).

## Bus events

- `spawn.recommended` — recommendation emitted (existing)
- `spawn.denied` — gate blocked autospawn (`code`, `reason` in payload)
- `spawn.executed` — spawn completed (`task_id`, `approved_via`, `kind`)
- `agent.spawned` / `agent.result` — Forum Romanum (existing)

## State

- `data/spawn-gate.state.json` — execution and denial history (48h trim)

## Bypass

Manual operator paths bypass the gate:

- `gzmo kurator approve <id>`
- `gzmo kurator fix-from-discovery --report ... --spawn`

## Verify

```bash
gzmo kurator status
tail -20 data/Synapse/events.jsonl | rg 'spawn\.(denied|executed|recommended)'
cargo test -p gzmo-core spawn_gate spawn_prime_budget --quiet
```
