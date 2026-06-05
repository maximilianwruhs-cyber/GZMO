# Tiered Memory session — 2026-06-05

## Summary
- Phases completed: **A (persist+link) + B (recall stream + eval)**
- Strict recall: **11/87 → 38/87 (43.7%)** — clears M0 (31/87), approaching M2 (44/87)
- Evidence rows: 1010 (ratio ≈ 0.50 to honeypot latest)
- Hits carrying evidence_text: 194/245
- Tests: `cargo build --release` clean; recall_rrf unit tests green (prior run)

## Substeps completed

| Step | Status | Notes |
|------|--------|-------|
| A.1 schema v5 | PASS | `evidence` + `evidence_fts`, `user_version=5` |
| A.2 types | PASS | `EvidenceSpan` on `ExtractedTruth` |
| A.3 localize | PASS | `evidence_localize.rs` quote→offset→±1 sentence window |
| A.4 ingest wire | PASS | `maybe_upsert_evidence` in promote (new + corroborate) |
| A.6 pilot | PASS | backup_custodian: joined T1/T2 rows confirmed |
| A.7 backfill | PARTIAL | 56/57 files; 1 fail (verifier JSON `-1` parse, unrelated) |
| B.1–B.3 streams | PASS | evidence FTS + vector fused into `recall_rrf` |
| B.4 MemoryHit | PASS | `evidence_text` populated via `get_evidence_text` |
| B.5 strict matcher | PASS | matches `evidence_text` first, falls back to content |
| B.6 A/B eval | PASS | 38/87 |

## Strict eval

```
Recall@5: 0.4368 (38/87 facts)
```

## Remaining 49 losses (Phase C worklist)

| Bucket | Count | Meaning | Fix |
|--------|-------|---------|-----|
| **E3** | 13 | Evidence span exists in store but ranked outside top-5 | Tune evidence stream weight / rescue |
| **E1/E2** | 36 | No matching evidence span in store | See note |

**E1/E2 note:** Most are from `sources*` / `quelltext_code_*` files that `qualifies_for_honeypot` **deliberately excludes** (code dumps, source scrapes). Those golden facts can never recall from the honeypot regardless of evidence — this is a golden-set vs honeypot-scope mismatch, not an evidence bug. A handful (e.g. `Live-Streaming von Energie-Metriken`, `Micro Virtual Machines (MicroVMs)`) are in includable docs where the verifier emitted no quote → candidates for per-observation evidence (Phase C.3).

**Realistic ceiling at current honeypot scope:** ~38 + 13 (E3 ranking-recoverable) = ~51/87, minus legitimately-excluded source files.

## Regressions
- None observed; recalled set grew from 11 to 38 (additive).

## Blocked / needs decision
- Phase C.3 (per-observation evidence) vs accepting source-file exclusions defines the path past ~44/87.
- Phase D recertify should wait until strict is stable across two runs.

## Commands run
- `cargo build --release -p gzmo-cli`
- `gzmo ingest <pilot>` then `gzmo ingest-dir <corpus>`
- `run-recall-eval.py --batch all --backend gzmo --match strict`
