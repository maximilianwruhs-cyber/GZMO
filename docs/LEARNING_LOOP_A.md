# Phase A learning loop — closure criteria (lab / GZMO-next)

**Status:** deepen dream/spark **wiring**, not a new little-tool organ.  
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
```

## Shipped deepens (2026-07-20)

1. **spark → vault** — `ltl-common` `vault-promote-spark` after cognition-smoke stage 4 (eligible when `promoted` / verdict supported; refuses `/opt/gzmo`)
2. **distill handoff promote** — `synapse-distill-handoff.sh --promote` runs `vault-promote-distill` into lab vault
3. Young-vault spark window — `SPARK_ANCHOR_WINDOW` / vault default `0,30`
4. **Fixture Phase A proof** — `spark-link/fixtures/phase-a-facts.json` (stale + ≤72h recent); dry-run `--allow-selection` writes selection audit; meta `learning_loop.phase_a_proof=true`
5. **`learning_loop` block** on `cognition-smoke-meta.json` (`spark_selected`, `vault_spark_promoted`, `phase_a_proof`)

## Live LLM proof

```bash
export LLM_URL=http://127.0.0.1:8000
# Reproducible promoted=true (skip flaky citation verify gate):
bash little-tools-lab/scripts/cognition-smoke.sh --fixture --spark-run --spark-no-verify \
  --meta /tmp/cognition-smoke-meta.json
# Expect: spark_report_promoted=true, phase_a_proof=true, spark_promote_kind=hypothesis

# Full verify (may abstain on citations; hypothesis still vault-audited via --allow-hypothesis):
bash little-tools-lab/scripts/cognition-smoke.sh --fixture --spark-run \
  --meta /tmp/cognition-smoke-meta.json
```

## Still open

1. Cross-link `dream-stats.json` into the same `learning_loop` block
2. Mature `data-next` vault path that selects under `SPARK_ANCHOR_WINDOW=14,90` without fixture
