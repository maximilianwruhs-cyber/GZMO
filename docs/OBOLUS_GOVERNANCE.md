# Obolus Governance

Runtime energy bilanz for Prime cognition. Complements ARCH-DIR (build/deploy sovereignty).

## Tier model

| Tier | Paths | Gate behavior |
|------|-------|---------------|
| **T0 Operator** | `gzmo chat`, TUI, `gzmo kurator approve` | Measure + `obolus.warn`; never hard deny |
| **T1 Semi-autonom** | Discovery timer cycle | Defer (skip cycle) |
| **T2 Autonom** | Kurator autospawn, discovery fixer, dream/spark/dice daemon | Hard deny + `obolus.denied` |

## Budget signals (Phase B)

- `E_total` — sum of all token usage in the rolling 1h window
- `ctx_%` — **max per-process** cumulative input / `prime_context_tokens` in that window (matches CLI report semantics; not summed across processes)
- `peak_call_ctx_%` — largest single-call input in the window (observability only)
- η (efficiency) — advisory only until Phase C; see [OBOLUS_EFFICIENCY.md](OBOLUS_EFFICIENCY.md)
- **Joules (observability)** — CPU RAPL + GPU watt integration in `power.jsonl`; see [OBOLUS_ENERGY.md](OBOLUS_ENERGY.md). Not used for gates until calibration.

## Config

```toml
[obolus_governance]
enabled = true
max_e_total_per_hour = 2000000
max_ctx_pressure_pct = 400
operator_warn_only = true
on_budget_exceeded = "deny"   # T1/T2 only
```

Autospawn requires **both** Redis `prime_spawn_budget` slots **and** Obolus allow.

## Synapse events

| Event | Meaning |
|-------|---------|
| `obolus.denied` | T1/T2 action blocked |
| `obolus.warn` | T0 over budget warning |
| `obolus.budget_tick` | Hourly `SystemBalance` snapshot |
| `obolus.energy_tick` | Hourly joules + `tokens_per_wh` correlation |
| `obolus.efficiency_tick` | Hourly η rollups |

## Operator runbook

```bash
gzmo obolus status
gzmo obolus balance              # rolling 1h SystemBalance for gates
gzmo obolus energy --since 1h    # hardware power.jsonl samples
gzmo obolus correlate --since 24h
gzmo obolus report --since 1h
gzmo obolus preflight discovery_cycle   # exit 0=allow, 1=deny, 2=defer
./scripts/sovereignty-verify.sh
```

When autospawn is denied: check ledger coverage (`docs/Obolus/instrumentation-map.md`), raise `max_e_total_per_hour`, or quiesce daemon loops.

Repeatable deny smoke (temporarily lowers E cap, restores config):

```bash
./scripts/obolus-gate-smoke.sh
```

## Instrumentation

Every Prime path must appear in the ledger before gates are considered fair. See [Obolus/instrumentation-map.md](Obolus/instrumentation-map.md).
