# Eval tiers — fast by default

**Problem:** Full `replay-wave.sh` (~18–45 min) is for baselines only.  
**Default:** `eval-quick.sh` (~30 s).

## Pyramid

| Tier | Command | Time | Prime? | When |
|------|---------|------|--------|------|
| **0** | `scripts/ingest-quality/eval-quick.sh` | ~30 s | No | Every change; YAML/docs/infra |
| **1** | `CORE=1 eval-quick.sh` or `replay-wave-core.sh` | ~5–8 min | Yes | `ingest_prep`, prompts, extract/verify |
| **2** | `replay-wave-batch2.sh` | ~15–25 min | Yes | Batch-2 golden expansion |
| **3** | `replay-wave.sh` + promote | ~18–45 min | Yes | Release / new baseline only |

## Tier 0 commands

```bash
scripts/ingest-quality/check-contract.sh
scripts/ingest-quality/gate-report.sh
python3 scripts/ingest-quality/retrieval-probes.py
scripts/check-fts-sanity.sh
```

## Tier 1 — 15 core golden files

```bash
scripts/ingest-quality/replay-wave-core.sh
# Merges into report.json; refreshes scoped rel-prom + contract fields
```

File list: `scripts/ingest-quality/core-golden-files.txt` (M4 Batch-1).

## Single-file Prime (~1–2 min)

```bash
python3 scripts/ingest-quality/patch-report-file.py <corpus-file>
```

## Missing facts (offline)

```bash
python3 scripts/ingest-quality/report-missing-facts.py
# → reports/missing-facts-YYYYMMDD.md
```

To raise fact recall without full replay:

1. `python3 scripts/ingest-quality/sharpen-golden-facts.py --only-missing --write` (align YAML to current `report.json`)
2. `fill-golden-from-report.py --write` for empty stubs
3. `N=5 scripts/ingest-quality/patch-worst-facts.sh` — Prime re-extract on top gaps (needs Prime)
4. `replay-wave-core.sh` after ingest logic changes

## Strict gate on demand

```bash
STRICT=1 scripts/ingest-quality/eval-quick.sh
```

## Related

- [EVAL_SCAFFOLD.md](./EVAL_SCAFFOLD.md)
- [M4_CONTINUOUS_EVAL_PLAN.md](./M4_CONTINUOUS_EVAL_PLAN.md)
- [BASELINE_STATUS.md](./BASELINE_STATUS.md)
