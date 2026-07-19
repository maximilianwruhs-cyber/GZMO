# Spine focus — two pillars

**Status:** Active (2026-07-19)  
**Supersedes for product direction:** expanding the opportunity-map zoo  
**Audience:** Operator deciding what to strengthen next

## Verdict

Stop expanding Arena / HSP / € / AOS / research stubs. Concentrate on two pillars:

1. **Living overnight metabolism** — chat → distill → vault/honeypot → recall you can feel  
2. **Product memory MCP** — `gzmo memory mcp` / `mcp-serve` as the attach surface for other agents

Everything else stays parked as nightburst spikes unless it directly protects or demos those pillars.

## Living vault owner

| Role | Host | Path | Overnight writer |
|------|------|------|------------------|
| **Living production** | **CT101** | `/opt/gzmo/` + `gzmo-daemon` | Yes — sole writer |
| **Operator / lab** | Workstation | `GZMO/data-next/` | No overnight `gzmo serve` while CT101 lives |

This is already ADR-0003 / [CT101_BOUNDARY.md](CT101_BOUNDARY.md). Nightburst artifacts under `data-next/` prove compressed recipes; they do **not** replace CT101 as the living brain.

## Keep / Park / Later

See the top of [STACK_OPPORTUNITY_MAP.md](STACK_OPPORTUNITY_MAP.md).

## Production readiness

```bash
# Laptop Memory MCP
bash scripts/product-readiness-gate.sh
# → data-next/product-readiness/latest.json

# Living CT101 metabolism
bash scripts/living-readiness-gate.sh
# → data-next/living-readiness/latest.json
```

See [PRODUCT_PRODUCTION_READINESS.md](PRODUCT_PRODUCTION_READINESS.md) and [LIVING_PRODUCTION_READINESS.md](LIVING_PRODUCTION_READINESS.md).

## Demable check (≈5 minutes)

```bash
# Stranger / laptop product path (no CT101 required)
bash scripts/product-stranger-path.sh
# → data-next/product-stranger/latest.{json,md}
bash scripts/mcp-attach-check.sh
# → data-next/mcp-attach/latest.{json,md}
bash scripts/product-first-fact.sh
# → data-next/product-first-fact/ (needs local engine; sibling overlay only)

# Soft CT101 living probe (SSH smoke + dual-writer check)
bash scripts/ct101-living-probe.sh
# → data-next/ct101-living/latest.{json,md}

# Living takeaway → distill → recall (CT101 same sitting)
bash scripts/ct101-takeaway-recall.sh
# → data-next/ct101-takeaway-recall/latest.{json,md}

# Faithfulness on living vault (CORE_INSIGHT / ADR claims)
bash scripts/faithfulness-living.sh
# → data-next/faithfulness-living/latest.{json,md}

# Lab supports (workstation data-next only)
bash scripts/takeaway-ritual-lab.sh   # enqueue only
bash scripts/dream-compact-lab.sh     # dry-run
bash scripts/spine-demo.sh
```

Stranger test: install → `product-stranger-path` → attach MCP in Cursor/Pi.  
Operator test: CT101 takeaway→recall HIT + living faithfulness + serve inactive.

## Related

- [PRODUCT_MCP.md](PRODUCT_MCP.md)
- [GZMO_NEXT_RUNBOOK.md](GZMO_NEXT_RUNBOOK.md)
- [CORE_INSIGHT.md](CORE_INSIGHT.md)
- [ADR-0003-one-instance-metabolism.md](ADR-0003-one-instance-metabolism.md)
