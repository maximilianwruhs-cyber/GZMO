# Obolus instrumentation map

Prime paths, process labels, and ObolusGate tiers. Update when adding a new LLM caller.

| Path | Process label | Tier | Instrumented |
|------|---------------|------|--------------|
| `GatewayRouter` / daemon engines | `dream_extract`, `spark_*`, `distill_*`, … | T2 | Yes |
| `gzmo chat` REPL | `chat` | T0 | Yes (`GatewayRouter` + `CallContextGuard`) |
| TUI agent | `chat` | T0 | Yes |
| `gzmo chaos skill` | `dice_loop` / skill name | T0/T2 | Yes (`headless_gateway`) |
| Kurator subagent | `kurator_*` | T2 | Yes |
| Dice daemon loop | `dice_loop` | T2 | Yes |
| Shell skill fallback | — | — | **Blocked** when `[obolus_governance] enabled` |
| Pi via reconcile | `pi_agent` | — | Yes (`synapse_pi`) |
| Reconcile energy sampler | `reconcile_sampler` | — | Yes (`power.jsonl`) |

## Dual signal (tokens + joules)

When `[obolus_analytics] energy_sampler_enabled = true`, the daemon reconcile loop appends hardware samples to `data/Obolus/power.jsonl` (CPU RAPL + GPU ∫P·dt). Gates still use **token** `E_total` only. See [OBOLUS_ENERGY.md](../OBOLUS_ENERGY.md).

## CallContextGuard

Operator paths should set `correlation_id` (session UUID) for attribution:

- `gzmo chat` — main session id
- Kurator subagent — Pi `correlation_id`

## Verification

```bash
gzmo obolus report --since 1h    # after a manual chat turn, expect `chat` process
./scripts/sovereignty-verify.sh  # check 12: instrumentation smoke
```
