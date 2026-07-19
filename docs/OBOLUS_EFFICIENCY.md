> **Recovered 2026-07-19** from `origin/feat/context-compress-headroom`. See [LOST_KNOWLEDGE_INVENTORY.md](./LOST_KNOWLEDGE_INVENTORY.md).

# Obolus Efficiency Analytics

Token ledger and context-window metrics for all LLM traffic on **Prime** (`:8000`).

Distinct from **[OBOLUS_ROUTING.md](OBOLUS_ROUTING.md)** — that document covers the static `TaskKind → engine` routing table. This document covers **token consumption** (`E_total`) and **context pressure** (`ctx_%`).

## Formula (roadmap)

η = (Q · I) / E_total — see wiki `efficiency-metric.md`. Phase A ships token ledger only; η comes in Phase B/C.

## Config

```toml
[obolus_analytics]
enabled = true
ledger_path = "data/Obolus/ledger.jsonl"
prime_context_tokens = 131072   # 128K — match start-prime PRIME_CTX
```

`[context_memory].context_length` is a **hot-budget** for scratch archive, not Prime `n_ctx`.

## CLI

```bash
gzmo obolus status              # today, top processes by E_total and ctx_%
gzmo obolus report --since 24h  # full table
gzmo obolus context --since 7d  # sorted by context_share_pct
gzmo obolus report --json       # machine-readable rollups
gzmo obolus efficiency --since 7d   # η = (Q·I)/E_total per process family
```

### Efficiency (Phase B/C)

| Process family | Q proxy | I proxy |
|----------------|---------|---------|
| dream | run produced KG/truths | kg_written / entities_extracted |
| spark | hypothesis promoted | links written (capped) |
| ingest | entities promoted | promoted / extracted |
| kurator_* | subagent status=done | tool_calls / llm_calls |
| pi_agent | quest_complete vs fail | output/input token ratio |

η/Mtok = η × 10⁶ for readable CLI scale.

Outcome sources: Synapse `dream.complete`, `spark.complete`, `ingest.complete`, `agent.result`, `quest_complete`/`quest_fail`.

Optional: `efficiency_tick_enabled = true` emits hourly `obolus.efficiency_tick` on the Synapse bus.

### Columns

| Column | Meaning |
|--------|---------|
| INPUT | Σ prompt tokens (context pressure) |
| OUTPUT | Σ completion tokens |
| TOTAL | E_total proxy (input + output) |
| CTX_% | Σ input / prime_context_tokens × 100 (can exceed 100% cumulatively) |
| PEAK_IN | max input_tokens in a single call |

## Ledger schema

Append-only JSONL at `data/Obolus/ledger.jsonl`. Fields align with OpenTelemetry GenAI conventions (`gen_ai.usage.input_tokens` / `output_tokens`).

Sources:

- `gateway` — instrumented GZMO `GatewayRouter` / `headless_gateway` calls
- `synapse_pi` — Pi `quest_complete` events (reconcile task)
- `llama_log` — unmatched llama-server log lines

## Deploy

```bash
./scripts/restart-daemon.sh --build
gzmo obolus status
```

Optional: set `llama_log_path` for external clients (Pi is covered via Synapse reconcile).
