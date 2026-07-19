# CT101 Cloud Routing (OpenRouter)

**Status:** Operator runbook (2026-07-19)  
**Mechanics (historical detail):** [CLOUD_MODE_DIAGNOSIS_2026-06-07.md](./CLOUD_MODE_DIAGNOSIS_2026-06-07.md)  
**Metering:** [ct101-systems/40-llm-gateway/obolus-metering.md](./ct101-systems/40-llm-gateway/obolus-metering.md)  
**Plans (scars):** `~/.cursor/plans/ct101_glm_5.2_cloud_*.plan.md`, `ct101_openrouter_verify_*.plan.md`

## Two switches (do not confuse)

| Config | Steers | Typical living intent |
|--------|--------|------------------------|
| `engine.active_mode` | Interactive chat engine selection | May be `cloud` on CT101 (GLM via OpenRouter) — **verify live toml** |
| `routing.cloud_first_background` | Dream / spark / ingest / distill / daemon background tasks | Cloud first, Prime failover when configured |

Daemon log `mode=local` from `active_mode` alone is **not** proof that background work is local.

## Living verify

```bash
ssh ct101 'grep -E "active_mode|cloud_first|openrouter|model" /opt/gzmo/gzmo.toml | head -40'
ssh ct101 'GZMO_CONFIG=/opt/gzmo/gzmo.toml /opt/gzmo/current/target/release/gzmo health'
# OpenRouter key must exist only in /opt/gzmo/.env — never in agent homes
```

Optional probe (repo, ignored by default):

```bash
cargo test -p gzmo-core --test live_cloud_probe -- --ignored --nocapture
```

## Failure modes

| Symptom | Likely cause |
|---------|--------------|
| HTTP **402** / credit errors | OpenRouter balance or key quota — not Prime down |
| Background stuck on Prime while expecting cloud | `cloud_first_background` false or cloud leaf misconfigured |
| Chat uses wrong model | Stale `~/.pi/agent/models.json` or MEMORY_REFERENCE routing table |
| Teach falls back to OpenRouter unexpectedly | Wrong `GZMO_CONFIG` / path pollution — [CT101_PATH_AUTHORITY.md](./CT101_PATH_AUTHORITY.md) |

## Model names

Pinned aliases change (Nemotron → GLM 5.2 + `reasoning_effort`, etc.). Trust **`/opt/gzmo/gzmo.toml` + `.env`**, not agent-home lore or the June 2026 diagnosis model strings.

## Relation to Obolus

Token spend and context pressure gates: [OBOLUS_GOVERNANCE.md](./OBOLUS_GOVERNANCE.md). Joules/RAPL are observability until calibrated: [OBOLUS_ENERGY.md](./OBOLUS_ENERGY.md).
