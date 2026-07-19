# Living-stack production readiness (CT101)

**Audience:** Sole overnight metabolism brain on CT101  
**Gate command:** `bash scripts/living-readiness-gate.sh`  
**GREEN:** exit `0` + `data-next/living-readiness/latest.json` → `"verdict": "GREEN"`

Laptop Memory MCP readiness is separate: [PRODUCT_PRODUCTION_READINESS.md](PRODUCT_PRODUCTION_READINESS.md).  
Historical workstation-centric checklist: [PRODUCTION_READINESS.md](PRODUCTION_READINESS.md) (superseded for living ops by this doc + ADR-0003).

## Definition of GREEN

1. **CT101** `gzmo-daemon` active with Redis / Qdrant / Neo4j sidecars Up  
2. **Vault** ≥ 10k facts; health probes OK (LLM, embed, qdrant, redis, neo4j, MCP memory, drift, distill queue)  
3. **Mentor** socket answers `pong`  
4. **No dual overnight writers** — workstation `gzmo-serve` inactive  
5. **Living faithfulness** — CORE_INSIGHT / ADR claims supported on CT101 vault  
6. **Takeaway → distill → recall** — same-sitting HIT on living vault  

## Gate checks

| Check | Severity |
|-------|----------|
| `dual-writer` | FAIL if workstation serve active |
| `ct101-living-smoke` | FAIL |
| `health:*` (cloud_llm, prime_llm, embeddings, qdrant, redis, neo4j, mcp_memory, drift, distill_queue) | FAIL if missing OK |
| `health:rerank` | HOLD if not OK |
| `vault-floor` | FAIL if &lt; 10k |
| `faithfulness-living` | FAIL |
| `takeaway-recall` | FAIL (skip with `LIVING_GATE_SKIP_TAKEAWAY=1`) |
| `workstation-prime` | HOLD if local `:8000` down but CT101 `prime_llm` OK |

## Operator commands

```bash
# Full living gate
bash scripts/living-readiness-gate.sh
# → data-next/living-readiness/latest.{json,md}

# Building blocks
bash scripts/ct101-living-smoke.sh
bash scripts/ct101-living-probe.sh
bash scripts/faithfulness-living.sh
bash scripts/ct101-takeaway-recall.sh

# Dual-writer hygiene (workstation)
systemctl --user stop gzmo-serve.service
systemctl --user disable gzmo-serve.service
```

## Related

- [CT101_BOUNDARY.md](CT101_BOUNDARY.md) — sole living instance  
- [CT101_RESTORE_LIVING.md](CT101_RESTORE_LIVING.md) — restore checklist  
- [ADR-0003-one-instance-metabolism.md](ADR-0003-one-instance-metabolism.md)  
- [SPINE_FOCUS.md](SPINE_FOCUS.md) — Keep pillars vs Park zoo  
