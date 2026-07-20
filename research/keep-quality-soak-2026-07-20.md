# Keep-quality soak — 2026-07-20

**USP:** full living on one airgapped box ([ADR-0004](../docs/ADR-0004-airgap-living-usp.md))  
**Gate:** `bash scripts/keep-quality-gate.sh` · soak: `bash scripts/keep-quality-soak.sh`

## Sample 1 (CT101 reference)

| Field | Value |
|-------|--------|
| Host | CT101 `/opt/gzmo` |
| When | 2026-07-20 (operator run) |
| Verdict | **GREEN** |
| Pass / fail / hold | 8 / 0 / 0 |

| Pillar | Result |
|--------|--------|
| living-readiness | PASS |
| felt-use | PASS — latest=38738 nonzero_recall=103 |
| spark-refractory | PASS — last_2 unique=2 |
| immune | PASS — candidates=0 |
| ripen | PASS — nonzero=103 core=32171 |
| night-lymph | PASS — night_id=2026-07-20 |
| mcp-attach | PASS — gzmo-living labeled |
| airgap-honesty | PASS — prime_llm + embeddings OK |

Artifact: `data-next/keep-quality/latest.json` (gitignored).  
Soak log: `data-next/keep-quality/soak-log.jsonl` — trailing_green=1 / need=3.

## Unpark gate

`keep-quality-soak.sh --summary` returns ready only after **3 trailing GREEN** samples (`KEEP_QUALITY_SOAK_NIGHTS=3`). Operators append one sample per night:

```bash
LIVING_GATE_SKIP_TAKEAWAY=1 bash scripts/keep-quality-soak.sh
bash scripts/keep-quality-soak.sh --summary
```

## Generalization

CT101 proves the living quality bar. Single-box airgap path (any host):

```bash
bash scripts/install-living-airgap.sh
# sole daemon writer + local MCP fragment → docs/AIRGAP_LIVING.md
bash scripts/keep-quality-gate.sh
```

Do not start Unpark Wave 1 brand expansion until soak summary advises `soak_ready_unpark_ok`.
