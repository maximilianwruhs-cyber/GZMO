# Phase A learning loop — closure criteria (lab / GZMO-next)

**Status:** **CLOSED** (2026-07-20) — dream/spark wiring + measurement complete; not a new little-tool organ.  
**Archaeology:** pick #6 (`75eb5004`, `6eaf665f`) — highest impact / lowest risk.  
**Boundary:** `data-next/` + LTL recipes only. Never CT101 vault import; no daemon `dice_loop`.

## Intended ring

```text
session-distill → honeypot-gate / promote → dream (01:00 recipe)
       → spark-link (select + optional verify)
       → vault / DREAMS.md feedback
       → next distill cycle (measurable)
```

## Closed when (lab proof)

| Check | Evidence |
|-------|----------|
| Dream metabolizes | `data-next/dream-stats.json` has `honeypot_promoted > 0` (or equivalent origin count) |
| Spark can select | `data-next/spark/last-spark-report.json` has non-null `selection` (not perpetual “No viable anchor”) |
| Young vaults | cognition-smoke vault path uses `--anchor-window 0,30` by default (`SPARK_ANCHOR_WINDOW` override) |
| Fixture still green | `bash little-tools-lab/scripts/cognition-smoke.sh --fixture` passes |
| Mid-band thick | census `midband_thick=true` (mid-band ≥24); thicken via `thicken-midband.py --apply` |
| Unified night | `scheduler-runs/latest-learning-loop.json` ties dream + spark under one `night_id` |

## Not Phase A closure

- Frontier hygiene (`forget-lint`, `verify-gates`, `token-economy`, `tool-chain`, `trace-memory`) — adjacent, not the dream/spark ring
- A new `learning-loop-A` sibling repo — wrong shape
- Grafting CT101 inline `SparkEngine` into next daemon

## Operator knobs

```bash
# Young lab vault — allow fresh-ish anchors
export SPARK_ANCHOR_WINDOW=0,30
bash little-tools-lab/scripts/cognition-smoke.sh --live --vault "$PWD/data-next/vault.db"

# Classic stale window (mature vaults)
export SPARK_ANCHOR_WINDOW=14,90

# Thicken organic mid-band (lab vault only; refuses /opt/gzmo)
python3 little-tools-lab/scripts/thicken-midband.py \
  --vault data-next/vault.db --target 48 --apply

# Night surface (dream 01:00 + cognition 03:30)
python3 little-tools-lab/scripts/learning-loop-night.py \
  --runs-dir data-next/scheduler-runs show --latest
```

## Shipped deepens (2026-07-20)

1. **spark → vault** — `ltl-common` `vault-promote-spark` after cognition-smoke stage 4 (eligible when `promoted` / verdict supported; refuses `/opt/gzmo`)
2. **distill handoff promote** — `synapse-distill-handoff.sh --promote` runs `vault-promote-distill` into lab vault
3. Young-vault spark window — `SPARK_ANCHOR_WINDOW` / vault default `0,30`
4. **Fixture Phase A proof** — `spark-link/fixtures/phase-a-facts.json` (stale + ≤72h recent); dry-run `--allow-selection` writes selection audit; meta `learning_loop.phase_a_proof=true`
5. **`learning_loop` block** on `cognition-smoke-meta.json` (`spark_selected`, `vault_spark_promoted`, `phase_a_proof`)
6. **dream-stats cross-link** — cognition-smoke auto-loads `data-next/dream-stats.json` (or `--dream-stats`); meta adds `dream_metabolized` + `phase_a_ring` (spark proof ∧ dream honeypot_promoted > 0)
7. **`--aged-vault`** — reproducible classic `14,90` selection without waiting on vault age
8. **Vault maturity census** — read-only `vault-maturity-census.py` → `classic_window_ready` / `midband_thick`
9. **Organic mid-band thicken** — `thicken-midband.py` accelerates oldest young `[3,14)d` rows into `[14,90]` (lab only)
10. **Unified night run-id** — `night_id` (UTC `YYYY-MM-DD`) on scheduler job records + `learning-loop-{night_id}.json` / `latest-learning-loop.json`; recipes export `GZMO_NIGHT_ID`

## Live LLM proof

```bash
export LLM_URL=http://127.0.0.1:8000
# Reproducible promoted=true (skip flaky citation verify gate):
bash little-tools-lab/scripts/cognition-smoke.sh --fixture --spark-run --spark-no-verify \
  --meta /tmp/cognition-smoke-meta.json
# Expect: spark_report_promoted=true, phase_a_proof=true, spark_promote_kind=hypothesis
# If data-next/dream-stats.json exists with honeypot_promoted>0 → phase_a_ring=true

# Full verify (may abstain on citations; hypothesis still vault-audited via --allow-hypothesis):
bash little-tools-lab/scripts/cognition-smoke.sh --fixture --spark-run \
  --meta /tmp/cognition-smoke-meta.json
```

## Mature 14,90 vault proof (lab seed)

```bash
bash little-tools-lab/scripts/cognition-smoke.sh --fixture --aged-vault \
  --meta /tmp/cognition-smoke-meta.json
# Seeds WORK/aged-spark-vault.db (honeypot rows at ~30–70d + ≤72h recent),
# runs spark dry-run with SPARK_ANCHOR_WINDOW=14,90 (default for this path).
# Expect: spark_mode=aged_vault_dry_run, spark_selected=true, phase_a_proof=true
```

## Organic maturity census (read-only)

```bash
python3 little-tools-lab/scripts/vault-maturity-census.py \
  --vault data-next/vault.db -o /tmp/vault-maturity.json
# cognition-smoke auto-attaches this into learning_loop when VAULT_PATH exists:
#   classic_window_ready, anchor_band_14_90, midband_thick, recent_lt_3d, …
```

Organic path is **ready** when `classic_window_ready=true` (≥1 latest honeypot in [14,90]d and ≥1 <3d). **Thick** when `midband_thick=true` (mid ≥24). Keep `--aged-vault` for reproducible CI without relying on vault age.

## Still open

None for Phase A. Next work is archaeology leftovers / other ladders — invent Phase B only if a new formal learning-loop ladder is needed.
