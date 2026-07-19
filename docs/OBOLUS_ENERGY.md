> **Recovered 2026-07-19** from `origin/feat/context-compress-headroom`. See [LOST_KNOWLEDGE_INVENTORY.md](./LOST_KNOWLEDGE_INVENTORY.md).

# Obolus Hardware Energy Bridge

Parallel telemetry for **physical energy** (CPU RAPL + GPU watt integration) alongside the existing **token proxy** `E_total`.

## Signals

| Signal | Source | Gate today? |
|--------|--------|-------------|
| `E_total` | Token ledger (`ledger.jsonl`) | Yes — `max_e_total_per_hour` |
| `ctx_%` | Token ledger | Yes — `max_ctx_pressure_pct` |
| `joules_cpu_1h` | RAPL via `power.jsonl` | No (observability) |
| `joules_gpu_est_1h` | HSP / nvidia-smi ∫P·dt | No (observability) |
| `tokens_per_wh` | `E_total / Wh_total` | No (calibration) |

Gates remain **token-only** until calibration experiments justify Wh-based limits.

## Config (`[obolus_analytics]`)

```toml
energy_sampler_enabled = true
power_ledger_path = "data/Obolus/power.jsonl"
rapl_energy_path = "/sys/class/powercap/intel-rapl:0/energy_uj"
energy_sample_min_interval_secs = 55
hsp_state_url = "http://127.0.0.1:8001/state"
nvidia_smi_fallback = true
gpu_joules_integration = true
```

RAPL sysfs must be readable:

```bash
sudo chmod a+r /sys/class/powercap/intel-rapl:0/energy_uj
```

## CLI

```bash
gzmo obolus balance              # E_total + joules side-by-side (1h)
gzmo obolus balance --json
gzmo obolus energy --since 1h    # raw power.jsonl samples
gzmo obolus correlate --since 24h
gzmo obolus sample               # one-shot sample (smoke / debug)
```

## Ledger schema (`power.jsonl`)

```json
{
  "ts": "2026-06-16T12:00:00Z",
  "source": "reconcile_sampler",
  "cpu_joules": 12.34,
  "cpu_joules_wh": 0.00343,
  "cpu_watts_avg": 45.2,
  "cpu_energy_source": "rapl",
  "gpu_power_w": 180.5,
  "gpu_joules_est": 3.01,
  "gpu_energy_source": "hsp",
  "sample_interval_s": 60.0,
  "host": "workstation"
}
```

`gpu_joules_est` is **trapezoid integration** of `power.draw` between samples — not NVML energy counters.

## Synapse events

| Event | Payload |
|-------|---------|
| `obolus.budget_tick` | `balance` includes new joule fields when sampler enabled |
| `obolus.energy_tick` | Hourly joules + `tokens_per_wh` |

## Calibration runbook (week 1–2)

1. **Idle baseline** — daemon on, no chat, 1h: expect `e_total ≈ 0`, `joules_wh > 0`
2. **Sustained chat** — 30min `gzmo chat`: note `tokens_per_wh` per hour
3. **Daemon cognition** — normal dream/spark/dice schedule: compare token rollups vs energy spikes

Decision matrix:

| Result | Action |
|--------|--------|
| Stable `tokens_per_wh` (±15%) | Keep token gates; Wh = reporting only |
| Poor token↔joule correlation | Consider Wh observability alerts; refine GPU ∫P·dt |
| Single-LLM, low autospawn | Simplify gates; keep experiment infra |

## Verification

```bash
./scripts/obolus-energy-smoke.sh
./scripts/sovereignty-verify.sh
```

## Related

- [OBOLUS_GOVERNANCE.md](OBOLUS_GOVERNANCE.md) — token tier model
- [Obolus/instrumentation-map.md](Obolus/instrumentation-map.md) — caller map
- Python offline experiments: `survey_Obolus` → `experiments.jsonl` (shared schema)
