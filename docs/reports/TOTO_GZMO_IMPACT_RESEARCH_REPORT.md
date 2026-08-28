# Toto-2.0-4m impact on GZMO — research report

**Date:** 2026-06-08  
**Model:** [Datadog/Toto-2.0-4m](https://huggingface.co/Datadog/Toto-2.0-4m) (zero-shot)  
**Protocol:** [`TOTO_GZMO_IMPACT_RESEARCH_BRIEF.md`](../TOTO_GZMO_IMPACT_RESEARCH_BRIEF.md)  
**Artifacts:** `experiments/toto-spike/` (`evaluate.py`, `results/*.tsv`)

---

## Executive verdict

### **IMPACT NO**

On **real** GZMO daemon telemetry (`chaos.rho_telemetry`), Toto-2.0-4m does **not** beat the best cheap baseline (**persistence**: \(\hat{\rho}_{t+h}=\rho_t\)) consistently or by the brief’s meaningful margin (≥15% MASE reduction). The shipped **EMA carry-forward** baseline is **worse** than persistence on real data — so Toto does not justify replacing or augmenting `rho_velocity_ema` today.

Task **T2** (saturation \(\rho_{\mathrm{mod}} > 6\)) is **untestable** on real logs: \(\rho_{\mathrm{mod}}\) never exceeds 0.0 in the captured window (507 Synapse samples). Synthetic lab CSVs either never cross the 6.0 threshold (`linear_decay_fast`) or have degenerate holdout windows already clamped at 10 (`baseline`).

Toto **does** show large MASE gains on **synthetic** `active_story_30s_linear_decay_fast` (36–68% vs persistence), but that regime (monotonic story-driven ramp toward ~6) does not match observed daemon behavior (quiet decay/recovery, \(\rho_{\mathrm{mod}} \in [-1.89, 0]\)).

**Operational fit passes:** CPU inference p50 ≈ **4.9 ms** (p90 ≈ 5.5 ms), model cache ≈ **16 MB**, air-gap path `experiments/toto-spike/.hf_cache/`.

**Recommendation:** **reject** integration. Re-open only after ≥24 h daemon capture that includes **saturation-adjacent** \(\rho_{\mathrm{mod}}\) dynamics (story bursts toward clamp) and T2 is scoreable on real holdout.

---

## Data summary

| Dataset | Label | N (events) | Tick span | \(\rho_{\mathrm{mod}}\) range | Saturation (>6) |
|---------|-------|------------|-----------|----------------------------------|-----------------|
| `data/Synapse/events.jsonl` | **real** | 507 | 15 → 7605 (Δt≈15 ticks/event) | \([-1.89, 0.00]\) | **none** |
| `active_story_30s_linear_decay_fast.csv` | synthetic | 766 | lab ticks | \([0, 5.99]\) | none |
| `active_story_30s_baseline.csv` | synthetic | 766 | lab ticks | \([0, 10]\) | yes (clamps); holdout plateau |

**Real data note:** Sample count exceeds the brief’s INCONCLUSIVE threshold (200), but the **regime is narrow** — no positive \(\rho_{\mathrm{mod}}\) impulses and no saturation events. Metrics are statistically usable for T1/T3 but not for the actionable T2 gate.

**Resampling:** Synapse events are already uniform at ~15 ticks (~5.2 s at 174 BPM). No resampling applied.

---

## Phase 5 decision matrix

| Task | Dataset | Horizon | Best baseline | Baseline metric | Toto-4m | Lift vs best baseline | Pass bar (≥15%)? |
|------|---------|---------|---------------|-----------------|---------|----------------------|------------------|
| **T1** \(\rho_{\mathrm{mod}}\) | real | 32 | persistence | MASE 10.28 | MASE 10.63 | **−3.4%** (worse) | No |
| **T1** \(\rho_{\mathrm{mod}}\) | real | 64 | persistence | MASE 14.21 | MASE 11.82 | **+16.8%** | Yes (one horizon) |
| **T1** \(\rho_{\mathrm{mod}}\) | real | 96 | persistence | MASE 11.79 | MASE 11.93 | **−1.2%** (worse) | No |
| **T1** \(\rho_{\mathrm{mod}}\) | lab fast | 96 | persistence | MASE 1.88 | MASE 1.19 | **+36.7%** | Yes (synthetic only) |
| **T2** saturate >6 | real | 96 | persistence | Brier 0.00† | Brier 0.00† | n/a | **Not scoreable** |
| **T2** saturate >6 | lab fast | 96 | persistence | Brier 0.00† | Brier 0.00† | n/a | **Not scoreable** |
| **T2** saturate >6 | lab baseline | 96 | persistence | Brier 0.00† | Brier 0.00† | n/a | **Not scoreable** |
| **T3** `rho_velocity_ema` | real | 96 | persistence | MASE 0.83 | MASE 0.83 | **0%** | No |

†All T2 labels are zero in evaluated holdout windows (no future crossings of 6.0); AUROC undefined.

### Baseline comparison detail (real, T1 @ H=96)

| Model | MASE | MAE | CRPS |
|-------|------|-----|------|
| persistence | **11.79** | 0.366 | — |
| linear_trend (k=8) | 67.37 | 2.092 | — |
| ema_carry_forward | 24.07 | 0.747 | — |
| **toto_4m** | 11.93 | 0.371 | 0.155 |

**Key finding:** The shipped EMA velocity signal does **not** produce competitive forecasts on real telemetry; **persistence wins**. Toto is effectively tied with persistence on the primary horizon, not clearly better than the cheapest baseline.

### Synthetic stress (lab `linear_decay_fast`, T1)

| Model | MASE @ H=96 |
|-------|-------------|
| persistence | 1.88 |
| ema_carry_forward | 51.84 |
| **toto_4m** | **1.19** |

EMA carry-forward **overshoots** on bursty ramp dynamics; Toto tracks better — but this scenario does not match real daemon ρ regime.

---

## Latency and footprint

| Item | Value |
|------|-------|
| Model weights | ~16 MB (`Datadog/Toto-2.0-4m`, safetensors) |
| HF cache (air-gap) | `experiments/toto-spike/.hf_cache/` |
| Python deps | `toto-models`, `torch` 2.12, `numpy` — venv at `experiments/toto-spike/.venv/` |
| CPU inference p50 | **4.9 ms** (H∈{32,64,96}, batch=1, context=128) |
| CPU inference p90 | **5.5 ms** |
| Daemon tick interval | ~345 ms (174 BPM) — **latency OK** (≪ tick) |

First model load adds one-time download from Hugging Face; subsequent runs are offline with cached weights.

---

## Gate evaluation (brief §5)

| Criterion | Result |
|-----------|--------|
| Toto wins **T2** | **Fail** — no positive labels in any holdout |
| Toto wins **T1 or T3** with clear margin on **real** data | **Fail** — mixed T1 (one horizon only); T3 no lift |
| CPU latency <100 ms | **Pass** (~5 ms) |
| ≥2 evaluation sets with signal | **Partial** — synthetic T1 only; real inconclusive for action |

**Brief gate for IMPACT YES:** not met.  
**Brief gate for IMPACT NO:** met — cheap baselines match or beat Toto on real T1; T2 untestable; no integration hook demonstrated on observed regime.

---

## Integration sketch

*Not recommended (IMPACT NO). If re-evaluated after richer telemetry:*

```
Synapse tail (chaos.rho_telemetry) → Python sidecar (toto-spike)
  → quantile forecast ρ_mod @ H=96
  → if p90(ρ_mod) > 6.0: HEARTBEAT alert + optional /stabilize suggestion
```

Max ~10 lines; **do not wire** until Phase-5 gate passes on **real** saturation-adjacent data.

---

## Recommendation

| Action | **reject** (close spike) |
|--------|--------------------------|
| Effort if revived | **M** — sidecar process, Synapse tail reader, alert plumbing |
| Preconditions | 24 h+ daemon run with story/crystallization load; confirm \(\rho_{\mathrm{mod}}\) excursions toward clamp; re-run `evaluate.py` |
| Keep | `experiments/toto-spike/` scripts for repeat eval; `.hf_cache` for air-gap |
| Drop | Any daemon/Synapse/gzmo-chaos wiring |

---

## Session checklist

| Phase | Status |
|-------|--------|
| 0 Preflight | ✅ 507 `chaos.rho_telemetry` rows; workspace `experiments/toto-spike/` |
| 1 Ingest | ✅ `ingest_synapse.py`, multivariate tensor |
| 2 Baselines | ✅ persistence, linear_trend, ema_carry_forward → `baseline_results.tsv` |
| 3 Toto spike | ✅ zero-shot load + score → `toto_results.tsv` |
| 4 Stress replay | ✅ lab CSVs (synthetic label) |
| 5 Gate | ✅ **IMPACT NO** |

---

## References

- [Toto-2.0-4m model card](https://huggingface.co/Datadog/Toto-2.0-4m)
- [Toto 2.0 paper](https://arxiv.org/abs/2605.20119)
- [`CHAOS_RHO_CONTROL_MODEL.md`](../CHAOS_RHO_CONTROL_MODEL.md) — observability fields, EMA \(\gamma=0.2\)
- Research brief: [`TOTO_GZMO_IMPACT_RESEARCH_BRIEF.md`](../TOTO_GZMO_IMPACT_RESEARCH_BRIEF.md)
