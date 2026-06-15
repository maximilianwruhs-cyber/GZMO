# Obolus Static Routing

Maps `TaskKind` to named engine profiles in `gzmo.toml` `[routing]`.
`GatewayRouter` resolves profiles to `Arc<dyn LlmGateway>` instances.

## Current steady state (Prime :8000)

| Task kind | Profile | Notes |
|-----------|---------|-------|
| `dream_extract`, `dream_verify` | `local` | Nightly REM |
| `spark_hypothesis`, `spark_verify` | `local` | Serendipity |
| `ingest_extract`, `ingest_verify` | `local_deterministic` | temp 0.1 |
| `distill_extract`, `distill_verify`, `distill_summary` | `local` | Session distill |
| `chat`, `pedagogy_internal`, `daemon` | `default_engine` (`local`) | Interactive |

## Kurator extensions (documented, optional)

```toml
[routing.mappings]
# agent_monitor = "local_deterministic"  # Kurator phase 1 summaries
# agent_spawn = "local"                  # Kurator phase 2 only (not wired)
```

Phase 1 Kurator uses heuristics only — no LLM `agent_monitor` calls yet.

## Not routed through Obolus

- `/dice` autopoietic loop (`headless_gateway` + chaos overrides)
- Mechanical skills (`/calculate`, etc.)

## Cost predictability

Static table buys predictable token spend before Kurator autospawn (phase 2).
See `gzmo-core/src/config.rs` `RoutingConfig::resolve`.
