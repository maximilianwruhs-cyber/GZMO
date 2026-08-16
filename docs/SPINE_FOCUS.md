# Spine focus — airgap living USP

**Status:** Active (2026-08-16) — **one product: living Keep** ([ADR-0007](ADR-0007-one-product-living.md))  
**Doctrine:** [ADR-0007](ADR-0007-one-product-living.md) (one SKU) · [ADR-0005](ADR-0005-flywheel-over-frozen-topology.md) (flywheel) · [ADR-0004](ADR-0004-airgap-living-usp.md) (airgap USP) · [ADR-0003](ADR-0003-one-instance-metabolism.md) (one writer)  
**Supersedes:** lite-as-bootstrap brand; co-primary “A + C”  
**Unpark roadmap:** [UNPARK_ROADMAP.md](UNPARK_ROADMAP.md)  
**Audience:** Operator deciding what to strengthen next  
**Quality gate:** `bash scripts/keep-quality-gate.sh` → `data-next/keep-quality/`

## Verdict

**Ship one local GZMO: the living Keep.** Former Park satellites are an **Unpark queue** — clients of living MCP after keep-quality soaks GREEN, not a second brand.

There is no lite product. `~/.gzmo` FTS MCP is an incomplete install or telescope scratch.

| | What it is |
|--|------------|
| **Product** | One box: `gzmo-daemon` + vault/honeypot + Redis + Qdrant + Neo4j + local Prime/embed + overnight metabolism + corpus search this sitting. MCP via stdio/localhost (`gzmo-living`). |
| **Clients** | Cursor / Pi / OpenClaw **attach** to that writer. They do not own a vault. |
| **Degraded** | Sidecars or embedder down — announce or fail closed. Not a SKU named lite. |

**Not primary brand:** Pi-as-UX (optional glass), Park zoo (Unpark queue), public webserver MCP, Telegram-as-brain.

## Hard boundary (do not blur)

Writer vs client is ops, not two products ([ADR-0007](ADR-0007-one-product-living.md)).

| | Client (hand) | Living writer (Keep) |
|--|--|--|
| Vault | None of its own (or telescope scratch) | `/opt/gzmo` (CT101 today) or appliance data dir |
| Sidecars | N/A | Redis + Qdrant + Neo4j **on the same box** (or honest degrade) |
| Overnight writer | No | Yes — sole writer ([ADR-0003](ADR-0003-one-instance-metabolism.md)) |
| MCP | stdio → living `mcp-serve` / owner socket | Owns the vault ([ADR-0006](ADR-0006-owner-control-plane.md)) |

Never a second overnight writer. Never market `gzmo-memory` on `~/.gzmo` as GZMO.

## Living vault owner

| Role | Host | Path | Overnight writer |
|------|------|------|------------------|
| **Living production (reference)** | **CT101** | `/opt/gzmo/` + `gzmo-daemon` | Yes when claim=`ct101` |
| **USP target** | Any one airgapped box | local data dir + compose pin | Yes when claim=`appliance` |
| **Dev living (allowed)** | Workstation | `GZMO/data-next/` or local vault | Yes when claim=`workstation` and CT101 writers stopped (`living-host-mutex.sh`) |

See [CT101_BOUNDARY.md](CT101_BOUNDARY.md), [AIRGAP_LIVING.md](AIRGAP_LIVING.md), [LIVING_APPLIANCE.md](LIVING_APPLIANCE.md).

**Workstation Neo4j is throwaway** — living smoke/auth SoT is the living host (CT101: `/opt/database-cluster/.env`).

## Migration vocabulary (lite / A / C)

Older docs said **lite** = product MCP, **A** = stranger MCP, **C** = living appliance. Map:

- **lite / A → client attach** (or incomplete install) — not a SKU
- **C → the product** (living Keep)

Do not invent new co-primary brands.

## Keep / Unpark / Later

See [STACK_OPPORTUNITY_MAP.md](STACK_OPPORTUNITY_MAP.md) and [UNPARK_ROADMAP.md](UNPARK_ROADMAP.md). Unpark only after keep-quality soaks GREEN.

**Active Unpark focus:** [BRAIN_FEED.md](BRAIN_FEED.md) — the only Unpark lane that claims to **nourish** the living vault (takeaway, tinyFolder, Felt Use, serendipity promote, calibration/Arena human-pin). Theater satellites stay sequenced but demoted.

**What to build next:** [OPPORTUNITY_DISCOVERY.md](OPPORTUNITY_DISCOVERY.md) — Sense→Rank→Bet→Ship→Soak (bet log under `research/opportunities/`). Customer-facing table stakes (offline corpus + hybrid) land on the living box, not on a lite twin.

## Production / quality readiness

```bash
# Product quality bar (living box)
bash scripts/keep-quality-gate.sh
# → data-next/keep-quality/latest.json

# Living ops readiness (CT101 reference)
bash scripts/living-readiness-gate.sh
# → data-next/living-readiness/latest.json

# Client attach smoke (not a second product GREEN)
bash scripts/product-readiness-gate.sh
# → data-next/product-readiness/latest.json

# Sidecar compose pin
bash scripts/living-appliance-up.sh
bash scripts/living-appliance-gate.sh
```

See [KEEP_QUALITY.md](KEEP_QUALITY.md), [LIVING_PRODUCTION_READINESS.md](LIVING_PRODUCTION_READINESS.md), [PRODUCT_PRODUCTION_READINESS.md](PRODUCT_PRODUCTION_READINESS.md).

## Demable check (≈5 minutes)

```bash
# Living reference (CT101) + USP quality
bash scripts/ct101-living-probe.sh
bash scripts/faithfulness-living.sh
bash scripts/keep-quality-gate.sh
```

Living test: keep-quality GREEN + one-writer + local MCP attach (`gzmo-living`). Incomplete `~/.gzmo` is not the demo.

## Related

- [ADR-0007-one-product-living.md](ADR-0007-one-product-living.md)
- [ADR-0004-airgap-living-usp.md](ADR-0004-airgap-living-usp.md)
- [AIRGAP_LIVING.md](AIRGAP_LIVING.md)
- [MCP_LOCAL_ATTACH.md](MCP_LOCAL_ATTACH.md)
- [PRODUCT_MCP.md](PRODUCT_MCP.md) (client attach / historical installer)
- [LIVING_APPLIANCE.md](LIVING_APPLIANCE.md)
- [CORE_INSIGHT.md](CORE_INSIGHT.md)
- [ADR-0003-one-instance-metabolism.md](ADR-0003-one-instance-metabolism.md)
