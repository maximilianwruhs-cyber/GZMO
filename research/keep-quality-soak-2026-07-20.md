# Keep-quality soak — 2026-07-20

**USP:** full living on one airgapped box ([ADR-0004](../docs/adr/ADR-0004-airgap-living-usp.md))  
**Gate:** `bash scripts/keep-quality-gate.sh` · soak: `bash scripts/keep-quality-soak.sh`

## Soak summary

| Field | Value |
|-------|--------|
| Host | CT101 `/opt/gzmo` |
| Samples | 3 |
| Trailing GREEN | **3** / need 3 |
| Advice | **`soak_ready_unpark_ok`** |

Note: all three samples were taken the same calendar day (two with living-readiness, one organs-only). Prefer one sample per overnight metabolism night going forward; this run unlocks the Unpark *gate check*, not a multi-night biology proof.

## Latest sample (3/3)

| Pillar | Result |
|--------|--------|
| living-readiness | PASS |
| felt-use | PASS — latest=38738 nonzero_recall=107 |
| spark-refractory | PASS — last_2 unique=2 |
| immune | PASS — candidates=0 |
| ripen | PASS — nonzero=107 core=32171 |
| night-lymph | PASS — night_id=2026-07-20 sparks=2 |
| mcp-attach | PASS — gzmo-living labeled |
| airgap-honesty | PASS — prime_llm + embeddings OK |

Artifact: `data-next/keep-quality/latest.json` (gitignored).  
Soak log: `data-next/keep-quality/soak-log.jsonl`.

## Unpark

```bash
bash scripts/keep-quality-soak.sh --summary
# → soak_ready_unpark_ok
```

Wave 1 surfaces remain **clients of local living MCP**, not alternate metabolisms ([UNPARK_ROADMAP.md](../docs/UNPARK_ROADMAP.md)).

## Generalization

```bash
bash scripts/install-living-airgap.sh
# sole daemon writer + local MCP fragment → docs/AIRGAP_LIVING.md
bash scripts/keep-quality-gate.sh
```
