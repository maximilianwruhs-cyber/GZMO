# Spine focus — two goals (A + C)

**Status:** Unpark active (2026-07-19) — A+C GREEN · satellites sequenced  
**Supersedes freeze:** “Park stays parked / do not expand”  
**Futures research:** [research/CT101_STACK_FUTURE_2026-07.md](research/CT101_STACK_FUTURE_2026-07.md)  
**Unpark roadmap:** [UNPARK_ROADMAP.md](UNPARK_ROADMAP.md)  
**Audience:** Operator deciding what to strengthen next  
**Combined gate:** `bash scripts/production-readiness-gate.sh` → `data-next/production-readiness/`

## Verdict

**Co-primary brand stays A + C.** Former Park satellites are an **Unpark queue** (sequenced waves) — useful, not the stranger install.

| Goal | Name | What ships |
|------|------|------------|
| **A** | **Product Memory MCP** | Stranger laptop: `install-gzmo.sh` → `~/.gzmo` → `gzmo mcp-serve` / `gzmo memory mcp`. Redis/Qdrant/Neo4j **off**. |
| **C** | **Living sidecar appliance** | One-writer host (today CT101): `gzmo-daemon` + Redis + Qdrant + Neo4j as a **preconfigured living stack** (in-repo compose pin → demable install). |

Metabolism (chat → distill → honeypot → felt recall) is the living asset **inside C**. MCP attach to that vault is the operator/living profile — never confuse it with A’s stranger home.

**Not primary brand (may still ship as satellites):** Pi-as-UX (B stays optional glass), Park zoo (D = Unpark queue, not the product identity).

## Hard boundary (do not blur)

| | A — Product MCP | C — Living appliance |
|--|-----------------|----------------------|
| Vault | `~/.gzmo` | `/opt/gzmo` (CT101) or future appliance data dir |
| Sidecars | Off | Redis + Qdrant + Neo4j **required** |
| Overnight writer | No (attach-only) | Yes — sole writer ([ADR-0003](ADR-0003-one-instance-metabolism.md)) |
| Stranger install | Yes | No — operator / sovereign host |

Never point product MCP at the living vault. Never require C’s sidecars for A’s install.

## Living vault owner

| Role | Host | Path | Overnight writer |
|------|------|------|------------------|
| **Living production** | **CT101** | `/opt/gzmo/` + `gzmo-daemon` | Yes — sole writer |
| **Operator / lab** | Workstation | `GZMO/data-next/` | No overnight `gzmo serve` while CT101 lives |

This is already ADR-0003 / [CT101_BOUNDARY.md](CT101_BOUNDARY.md). Nightburst artifacts under `data-next/` prove compressed recipes; they do **not** replace CT101 as the living brain.

**Workstation Neo4j is throwaway** — living smoke/auth SoT is CT101 (`/opt/database-cluster/.env`). See [LIVING_APPLIANCE.md](LIVING_APPLIANCE.md#auth-neo4j).

## Keep / Unpark queue / Later

See [STACK_OPPORTUNITY_MAP.md](STACK_OPPORTUNITY_MAP.md) and [UNPARK_ROADMAP.md](UNPARK_ROADMAP.md).

## Production readiness

```bash
# Laptop Memory MCP
bash scripts/product-readiness-gate.sh
# → data-next/product-readiness/latest.json

# Living CT101 metabolism
bash scripts/living-readiness-gate.sh
# → data-next/living-readiness/latest.json

# Goal C compose pin (sidecars; no daemon)
bash scripts/living-appliance-up.sh
bash scripts/living-appliance-gate.sh
# → data-next/living-appliance/latest.json
# Living MCP label: bash scripts/install-shared-mcp.sh  →  gzmo-living
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
bash scripts/watchdog-lab.sh          # soft STALE under short threshold
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
