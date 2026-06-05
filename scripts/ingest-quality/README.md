# Ingest quality harness

Wave-1 eval gate for golden corpus (`gzmo_obolus`, 57 files). Blocks re-ingest until metrics pass.

## Stable scaffold (two layers)

| Layer | Command | When |
|-------|---------|------|
| **Contract** (deterministic) | `check-contract.sh` | After `expected.yaml` edits |
| **Pipeline** (LLM) | `replay-wave.sh` | Before purge / re-ingest |

See [docs/EVAL_SCAFFOLD.md](../../docs/EVAL_SCAFFOLD.md).  
**Antigravity handoff:** [ANTIGRAVITY_STATUS.md](../../docs/ANTIGRAVITY_STATUS.md) · [Step 4 flakes](../../docs/ANTIGRAVITY_STEP4_M1_FLAKES.md) · [Step 5 honeypot](../../docs/ANTIGRAVITY_STEP5_M2_HONEYPOT.md)

| File | Purpose |
|------|---------|
| `gate-config.yaml` | Thresholds + `layered` / `strict` mode |
| `gate-report.sh` | Gate table from `report.json` |
| `check-contract.sh` | Golden rescore only (~1s) + MemScore one-liner |
| `rescore-golden.py` | Offline contract scorer |
| `mem-score.py` | MemScore: recall tracks + dual faithfulness (`faith_context` gate / `faith_corpus`), composite |
| `faithfulness-judge.py` | M4 retrieval faithfulness: `--grounding context|corpus|both` ([M4_FAITHFULNESS_JUDGE.md](../../docs/M4_FAITHFULNESS_JUDGE.md)) |
| `validate-golden-facts.py` | Golden contract gate + corpus/LLM audit ([M4_GOLDEN_CONTRACT.md](../../docs/M4_GOLDEN_CONTRACT.md)) |
| `eval_llm.py` | Shared Prime client + verify rules + quote/snippet helpers (judge + validator) |
| `run-recall-eval.py` | Recall@5 eval; `--match normalized|token|strict`; `--track rrf|rrf_strict|golden|qdrant` |
| `../../docs/M4_MEMSCORE_RECALL5.md` | Recall@5 RAG eval spec (`recall@5=null` OK for end-gate) |
| `expected.yaml` | 15+35 golden files (15 baseline + 35 M4 stubs) |
| `eval-quick.sh` | Tier 0 (+ optional `CORE=1`) |
| `replay-wave-core.sh` | 15-file Prime eval → merge `report.json` |
| `merge-report-partial.py` | Merge partial eval into full report |
| `report-missing-facts.py` | Offline gap report for golden facts |
| `core-golden-files.txt` | Batch-1 file list for core replay |
| `replay-wave.sh` | Build, full `ingest-eval`, archive, gate |
| `report.json` | Latest dry-run output |
| `reports/run-*.json` | Archived reports (best-of pool) |
| `pipeline-lock.json` | **`baseline-m4-post-sprint`** lock (see [BASELINE_STATUS.md](../../docs/BASELINE_STATUS.md)) |
| `promote-baseline.sh` | Promote `report.json` → `reports/baseline-m4-current.json` + lock (no Prime) |
| `reports/baseline-m4-current.json` | Frozen eval report (SoT) |
| `reports/baseline-manifest.json` | Promote history |
| `baseline-2026-06-02.json` | Pre-purge store counts |
| `retrieval-probes.py` | Post-ingest RAG check (default Qdrant `honeypot`; `QDRANT_COLLECTION=knowledge` legacy) |
| `patch-report-file.py` | Re-eval one corpus file and refresh `report.json` summary |
| `../memory-status.sh` | Vault / honeypot / Qdrant point counts |
| `../check-fts-sanity.sh` | Honeypot vs FTS row parity; no broken triggers |
| `replay-wave-batch2.sh` | Dry-run eval on 20 M4 Batch-2 golden files → `report-batch2.json` |
| `recalc-pipeline-summary.py` | Scoped rel-prom waivers (`gate-config` patterns + `expected.yaml`) |
| `fill-golden-from-report.py` | Fill empty `expected.yaml` stubs from latest `report.json` (no Prime) |
| `sharpen-golden-facts.py` | Replace truncated/broken `must_fact_substrings` with in-text anchors |
| `../pre-ingest-gate.sh` | Stage-1/2 file validation before live ingest |
| `../gate-wave1-before-ingest.sh` | Wave-1 corpus gate + `wave1-ingest-ready.manifest` |
| `live-ingest-smoke.sh` | One-file live ingest — Neo4j MCP write path (opt-in via certify) |
| `certify-production-baseline.sh` | Full M4 certification + optional baseline promote |

**Architecture:** [CEILING_ROADMAP.md](../../docs/CEILING_ROADMAP.md) · [MEMORY_ARCHITECTURE_SPEC.md](../../docs/MEMORY_ARCHITECTURE_SPEC.md)

## Gate independence (F20)

| Command | Scope | Blocks on |
|---------|-------|-----------|
| `./scripts/verify-production.sh` | Infra liveness | Prime, embed, Neo4j MCP, daemon, vault, FTS sanity |
| `eval-quick.sh` | Offline + probes (~30s) | Frozen `report.json` contract, gate-report, retrieval-probes |
| `certify-production-baseline.sh` | M4 sign-off (~15–25 min) | Build, golden audit, extraction snapshot contract, strict recall floor, faithfulness_context, eval-quick STRICT=1 |
| `live-ingest-smoke.sh` | One-file live ingest (opt-in) | Neo4j MCP write path + vault promote (`LIVE_INGEST_SMOKE=1` in certify) |

**Dry-run vs live:** `gzmo ingest-eval` never writes vault/honeypot/evidence/Neo4j. Strict recall and faithfulness judge require a live store (re-ingest or backfill after pipeline changes).

## Quick commands

```bash
# Daily eval (~30s) — see docs/EVAL_TIERS.md
scripts/ingest-quality/eval-quick.sh

# Prime: 15 core golden only (~5–8 min), merges into report.json
scripts/ingest-quality/replay-wave-core.sh

# Missing golden facts (offline)
python3 scripts/ingest-quality/report-missing-facts.py

# YAML-only (uses latest report.json)
scripts/ingest-quality/check-contract.sh

# Full baseline eval (~18–45 min)
scripts/ingest-quality/replay-wave.sh

# Gate only
scripts/ingest-quality/gate-report.sh

# MemScore (standalone)
python3 scripts/ingest-quality/mem-score.py
python3 scripts/ingest-quality/mem-score.py --verbose

# M4 faithfulness pipeline (golden gate -> strict recall -> dual judge)
python3 scripts/ingest-quality/validate-golden-facts.py --fail-on-invalid
python3 scripts/ingest-quality/run-recall-eval.py --batch all --backend gzmo --match strict
python3 scripts/ingest-quality/faithfulness-judge.py --mode llm --grounding both --write-report --merge-mem-score

# Pre-ingest file validation
scripts/pre-ingest-gate.sh <file-or-directory>
scripts/pre-ingest-gate.sh <directory> --dry-run --quarantine

# Backfill SQLite honeypot (from root)
python3 scripts/backfill-honeypot.py

# Sync honeypot facts to Qdrant
python3 scripts/sync-vault-to-qdrant.py --source honeypot --collection honeypot
```

