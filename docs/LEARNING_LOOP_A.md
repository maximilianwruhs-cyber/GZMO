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

## Still open (next deepens)

1. spark-link verified hypotheses → vault/`graph_rel` write (lab promote bin)
2. scheduled `synapse-distill-handoff` promote tail when dream skipped
3. Unified `learning_loop` block across dream-stats + cognition-smoke-meta
