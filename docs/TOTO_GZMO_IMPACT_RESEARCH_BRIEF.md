# Research brief — Toto-2.0-4m impact on GZMO

**Audience:** Agent starting a fresh session.  
**Goal:** Decide whether [Datadog/Toto-2.0-4m](https://huggingface.co/Datadog/Toto-2.0-4m) has **real, justified impact** on GZMO — or conclude **no** and stop.  
**Default stance:** **Skeptical null hypothesis** — Toto is interesting but **not integrated** until evidence beats cheap baselines on GZMO-shaped data.

---

## Session opener (paste this to the agent)

```
You are a research bot evaluating whether Datadog Toto-2.0-4m has real impact on GZMO.

Read and follow: docs/TOTO_GZMO_IMPACT_RESEARCH_BRIEF.md (this file).

Rules:
- Falsifiable verdict only: IMPACT YES | IMPACT NO | INCONCLUSIVE.
- Do not wire Toto into daemon, Synapse, or gzmo-chaos until Phase 5 gate passes.
- Prefer existing GZMO telemetry; synthetic data only if real logs are insufficient (label clearly).
- Air-gapped friendly: offline model cache OK; no required cloud inference.
- Minimize scope: spike in an isolated folder, not production crates.

Deliver: docs/TOTO_GZMO_IMPACT_RESEARCH_REPORT.md with metrics tables, verdict, and next-step recommendation (integrate / defer / reject).
```

---

## 1. What GZMO is (relevant slice)

GZMO daemon runs `PulseLoop` (174 BPM) and exports ρ homeostasis telemetry:

| Source | Path | Fields (examples) |
|--------|------|-------------------|
| Live snapshot | `data/CHAOS_STATE.json` | `tick`, `mutations.lorenz_rho_mod`, `rho_effective`, `rho_mod_delta`, `rho_forcing_sign`, `rho_breath_phase`, `rho_velocity_ema`, `tension`, `energy` |
| Synapse bus | `data/Synapse/events.jsonl` | `event_type: chaos.rho_telemetry` every **15 ticks** (daemon only) |
| Lab ground truth | `~/Projects/chaos-breathing-lab/output/matrix/` | Scenario CSVs; `active_story_30s` is the stress case |

**Control law (already shipped):** linear `rho_decay_k=0.001`, optional tanh `rho_restore_alpha/beta`, `/stabilize`, EMA breath. See `docs/CHAOS_RHO_CONTROL_MODEL.md`.

**Toto is not a controller.** It is only worth pursuing if it **predicts** ρ (or related observability signals) **better than trivial baselines** in a way that enables a concrete GZMO action (early stabilize, alert, tuning).

---

## 2. Null hypothesis & decision rubric

### Null hypothesis (H₀)

> Toto-2.0-4m does **not** improve forecast accuracy enough on GZMO ρ telemetry (vs naive/EMA baselines) to justify integration cost (PyTorch sidecar, ops, air-gap model cache).

### “Real impact” (H₁) — **all** must be plausible after evaluation

| Criterion | Bar |
|-----------|-----|
| **Predictive lift** | Beats best baseline on held-out windows (see §4) by a **meaningful margin** (suggest: ≥15% CRPS or MASE reduction, or p90 saturation hit-rate +10pp) |
| **Actionable horizon** | Forecast at horizon H (ticks or seconds) that is **earlier than reaction from EMA alone** — e.g. “ρ_mod > 6 within next ~2 min” |
| **Operational fit** | 4m model runs on target hardware (CPU OK) within latency budget (<< daemon tick interval; target <100 ms inference on CPU for spike) |
| **Integration clarity** | One concrete hook identified (Synapse tail → forecast → trigger/HEARTBEAT row) without rewriting chaos physics |

### Verdict labels

| Verdict | Meaning |
|---------|---------|
| **IMPACT YES** | H₁ supported on ≥2 evaluation sets (real + synthetic or two real regimes); recommend a **minimal** Phase-6 spike PR scope |
| **IMPACT NO** | Baselines win or lift negligible; **stop** — document and close |
| **INCONCLUSIVE** | Too little data, install failure, or ambiguous metrics; list **exactly** what to collect next (duration, scenario) |

**If IMPACT NO → that is a successful session.** Do not integrate out of enthusiasm.

---

## 3. What Toto is (evaluation target)

- **Model:** [Datadog/Toto-2.0-4m](https://huggingface.co/Datadog/Toto-2.0-4m) — 4M params, ~16 MB, observability-oriented TS foundation model.
- **Install:** `pip install toto-models` → `from toto2 import Toto2Model` (see model card Quick Start).
- **I/O:** `(batch, n_variates, time_steps)` context → quantile forecasts `(9, batch, n_variates, horizon)`.
- **License:** Apache-2.0 (OK for evaluation spike).

**Do not** start with 22m/313m unless 4m shows signal and you need a sensitivity check.

---

## 4. Evaluation design

### 4.1 Forecast tasks (pick at least two)

| Task ID | Target | Question |
|---------|--------|----------|
| **T1** | `rho_mod` (or `rho_effective`) | Point/quantile forecast at H=32, 64, 96 steps ahead |
| **T2** | **Saturation event** | Binary: will `rho_mod` exceed **6.0** (or 7.0) any time in next H steps? |
| **T3** | `rho_velocity_ema` | Does multivariate context help predict EMA trend? |

Map Synapse samples to uniform time index: **Δt ≈ 15 ticks** per event (~5.2 s at 174 BPM). Document resampling if using `CHAOS_STATE.json` at different cadence.

### 4.2 Baselines (must implement — cheap)

Beat **all** of these or verdict is likely NO:

1. **Persistence:** \(\hat{y}_{t+h} = y_t\)
2. **Linear trend:** last-k slope extrapolation (k=8 Synapse points)
3. **EMA carry-forward:** use shipped `rho_velocity_ema` — \(\hat{\rho}_{t+h} = \rho_t + h \cdot v_t\) (γ from `gzmo.toml` `rho_ema_gamma=0.2`)
4. **Lab oracle (synthetic only):** replay `chaos-breathing-lab` CSV — known ground truth for sanity check

### 4.3 Metrics

| Metric | Use |
|--------|-----|
| **MASE** | Scale-free vs naive seasonal/persistence |
| **CRPS** | Compare Toto quantiles vs baseline point (pinball / CRPS approximation) |
| **Event Brier / AUROC** | T2 saturation classification at p50/p90 |
| **Lead time** | Minutes before clamp/saturation that alert fires (Toto vs EMA) |

### 4.4 Data splits

- **Real:** tail of `data/Synapse/events.jsonl` filtered to `chaos.rho_telemetry` (if <200 points, note INCONCLUSIVE risk).
- **Synthetic stress:** generate from `chaos-breathing-lab` `active_story_30s_linear_decay_fast.csv` or run lab sim — **label as synthetic**.
- **Split:** last 20% time holdout; no shuffle (time series).

---

## 5. Phased execution checklist

### Phase 0 — Preflight (30 min)

- [ ] Confirm daemon was running and Synapse has `chaos.rho_telemetry` rows (`grep chaos.rho_telemetry data/Synapse/events.jsonl | wc -l`)
- [ ] Read `docs/CHAOS_RHO_CONTROL_MODEL.md` §5 (observability fields)
- [ ] Create isolated workspace: `survey_GZMO/experiments/toto-spike/` (gitignore large weights if needed)

### Phase 1 — Data ingest (1 h)

- [ ] Script: Synapse JSONL → multivariate tensor `(n_variates, T)` with columns at minimum: `rho_mod`, `rho_velocity_ema`, `rho_mod_delta`
- [ ] Script: optional `CHAOS_STATE.json` time series parser (if denser than Synapse)
- [ ] Report: sample count, time span, missing gaps

### Phase 2 — Baselines (1 h)

- [ ] Implement T1–T3 baselines in Python or Rust (Python OK for spike)
- [ ] Score on holdout → table `baseline_results.tsv`

### Phase 3 — Toto spike (2 h)

- [ ] `pip install toto-models` (+ `torch`) in venv under `experiments/toto-spike/`
- [ ] Download `Datadog/Toto-2.0-4m` once (HF cache); note air-gap path for operator
- [ ] Minimal `forecast_rho.py`: load context window (≥128 steps if available, else pad), horizons 32/64/96
- [ ] Extract p50 / p90 for `rho_mod`; score CRPS/MASE vs baselines
- [ ] Log CPU inference latency (single forecast)

### Phase 4 — Stress replay (1 h)

- [ ] Run same protocol on lab CSV `active_story_30s` (synthetic label)
- [ ] Compare: does Toto see saturation coming earlier than EMA on story bursts?

### Phase 5 — Decision gate (30 min)

Fill decision matrix in report:

```markdown
| Task | Best baseline | Toto-4m | Lift? |
|------|---------------|---------|-------|
| T1 rho_mod @ H=96 | | | |
| T2 saturate>6 | | | |
| T3 velocity EMA | | | |
```

**Gate rules:**

- **IMPACT YES** if Toto wins T2 **and** at least one of T1/T3 with clear margin, plus CPU latency OK.
- **IMPACT NO** if EMA baseline within 5% of Toto on all tasks, or Toto fails to load/run reliably.
- **INCONCLUSIVE** if real Synapse rows < 200 — specify “run daemon 24h with telemetry, re-evaluate.”

---

## 6. Deliverables (required)

Write **`docs/TOTO_GZMO_IMPACT_RESEARCH_REPORT.md`** containing:

1. **Executive verdict** (YES / NO / INCONCLUSIVE) — one paragraph
2. **Data summary** (real vs synthetic, N, span)
3. **Results tables** (baselines vs Toto per task/horizon)
4. **Latency & footprint** (model MB, inference ms, deps)
5. **Integration sketch** (only if YES) — max 10 lines: Synapse tail → sidecar → action
6. **Recommendation** — integrate | defer | reject, with effort estimate (S/M/L)

Optional artifacts under `experiments/toto-spike/`:

- `ingest_synapse.py`, `forecast_rho.py`, `results/*.tsv`

**Do not** commit HF weights or venv to git.

---

## 7. Explicit out of scope (this research session)

- Wiring Toto into `gzmo-daemon` / `chaos_bootstrap.rs`
- Replacing `rho_velocity_ema` or tanh governor with ML
- Fine-tuning Toto on GZMO data (zero-shot only for this brief)
- Multi-model sweep (22m+) unless 4m YES and reviewer asks
- Building a Hugging Face Space or UI

---

## 8. References

| Resource | URL |
|----------|-----|
| Toto-2.0-4m model card | https://huggingface.co/Datadog/Toto-2.0-4m |
| Toto 2.0 paper | https://arxiv.org/abs/2605.20119 |
| GZMO ρ control model | `docs/CHAOS_RHO_CONTROL_MODEL.md` |
| ρ implementation handoff | `docs/CHAOS_RHO_IMPLEMENTATION_HANDOFF.md` |
| Chaos breathing lab | `~/Projects/chaos-breathing-lab/` |
| Synapse export code | `gzmo-cli/src/chaos_bootstrap.rs` (SenseChaosRho) |

---

## 9. Success definition for the session

A **good** session ends with a clear **NO** and a saved report — same as a clear **YES**.

Avoid: “Toto seems cool, we could integrate later” without numbers.

---

*Created 2026-06-08 for next-session research bot. Update this brief if evaluation protocol changes.*
