# Spine focus — airgap living USP

**Status:** Active (2026-07-20) — USP = full living on one airgapped box  
**Doctrine:** [ADR-0005](ADR-0005-flywheel-over-frozen-topology.md) (flywheel) · [ADR-0004](ADR-0004-airgap-living-usp.md) (USP) · one-writer [ADR-0003](ADR-0003-one-instance-metabolism.md)  
**Supersedes:** co-primary “A + C” brand forever (kept below as migration vocabulary only)  
**Unpark roadmap:** [UNPARK_ROADMAP.md](UNPARK_ROADMAP.md)  
**Audience:** Operator deciding what to strengthen next  
**Quality gate:** `bash scripts/keep-quality-gate.sh` → `data-next/keep-quality/`

## Verdict

**Develop living. Ship one local GZMO.** Former Park satellites are an **Unpark queue** — clients of local living MCP after keep-quality soaks GREEN, not the stranger brand.

| Profile | Name | What ships |
|---------|------|------------|
| **Living** (first-class) | **Airgap living Keep** | One box: `gzmo-daemon` + vault/honeypot + Redis + Qdrant + Neo4j + local Prime/embed + overnight metabolism. MCP via stdio/localhost. |
| **Lite** (bootstrap only) | **Attach Memory MCP** | `install-gzmo.sh` → `~/.gzmo` → `gzmo mcp-serve`. Sidecars **off**. No overnight writer. |

Metabolism (chat → distill → honeypot → felt recall → dream/spark/immune/lymph) lives **only** on the living profile. Lite must never become a second overnight brain.

**Not primary brand:** Pi-as-UX (optional glass), Park zoo (Unpark queue), public webserver MCP.

## Hard boundary (do not blur)

| | Lite (bootstrap) | Living (USP) |
|--|------------------|--------------|
| Vault | `~/.gzmo` | `/opt/gzmo` (CT101 today) or any single-box appliance data dir |
| Sidecars | Off | Redis + Qdrant + Neo4j **on the same box** (or honest degrade) |
| Overnight writer | No | Yes — sole writer ([ADR-0003](ADR-0003-one-instance-metabolism.md)) |
| MCP attach | stdio → local `mcp-serve` | stdio → local `mcp-serve` (SSH wrap is ops, not brand) |
| Roadmap weight | Maintenance | **First-class** |

Never point lite product MCP at the living vault as the stranger default. Never require living sidecars for lite bootstrap. Never dual overnight writers.

## Living vault owner

| Role | Host | Path | Overnight writer |
|------|------|------|------------------|
| **Living production (reference)** | **CT101** | `/opt/gzmo/` + `gzmo-daemon` | Yes when claim=`ct101` |
| **USP target** | Any one airgapped box | local data dir + compose pin | Yes when claim=`appliance` |
| **Dev living (allowed)** | Workstation | `GZMO/data-next/` or local vault | Yes when claim=`workstation` and CT101 writers stopped (`living-host-mutex.sh`) |

See [CT101_BOUNDARY.md](CT101_BOUNDARY.md), [AIRGAP_LIVING.md](AIRGAP_LIVING.md), [LIVING_APPLIANCE.md](LIVING_APPLIANCE.md).

**Workstation Neo4j is throwaway** — living smoke/auth SoT is the living host (CT101: `/opt/database-cluster/.env`).

## Migration vocabulary (A / C)

Older docs said **A** = product MCP and **C** = living appliance. Map:

- **A → lite profile** (bootstrap)
- **C → living profile** (USP)

Do not invent new co-primary brands. Prefer “lite / living.”

## Keep / Unpark / Later

See [STACK_OPPORTUNITY_MAP.md](STACK_OPPORTUNITY_MAP.md) and [UNPARK_ROADMAP.md](UNPARK_ROADMAP.md). Unpark only after keep-quality soaks GREEN.

**Active Unpark focus:** [BRAIN_FEED.md](BRAIN_FEED.md) — the only Unpark lane that claims to **nourish** the living vault (takeaway, tinyFolder, Felt Use, serendipity promote, calibration/Arena human-pin). Theater satellites stay sequenced but demoted.

**What to build next:** [OPPORTUNITY_DISCOVERY.md](OPPORTUNITY_DISCOVERY.md) — Sense→Rank→Bet→Ship→Soak (bet log under `research/opportunities/`).

## Production / quality readiness

```bash
# USP quality bar (living box — preferred)
bash scripts/keep-quality-gate.sh
# → data-next/keep-quality/latest.json

# Living ops readiness (CT101 reference)
bash scripts/living-readiness-gate.sh
# → data-next/living-readiness/latest.json

# Lite bootstrap (optional / stranger day-zero)
bash scripts/product-readiness-gate.sh
# → data-next/product-readiness/latest.json

# Sidecar compose pin
bash scripts/living-appliance-up.sh
bash scripts/living-appliance-gate.sh
```

See [KEEP_QUALITY.md](KEEP_QUALITY.md), [LIVING_PRODUCTION_READINESS.md](LIVING_PRODUCTION_READINESS.md), [PRODUCT_PRODUCTION_READINESS.md](PRODUCT_PRODUCTION_READINESS.md).

## Demable check (≈5 minutes)

```bash
# Lite bootstrap (no living host required)
bash scripts/product-stranger-path.sh
bash scripts/mcp-attach-check.sh

# Living reference (CT101) + USP quality
bash scripts/ct101-living-probe.sh
bash scripts/faithfulness-living.sh
bash scripts/keep-quality-gate.sh
```

Lite test: install → attach MCP in Cursor/Pi.  
Living test: keep-quality GREEN + one-writer + local MCP attach.

## Related

- [ADR-0004-airgap-living-usp.md](ADR-0004-airgap-living-usp.md)
- [AIRGAP_LIVING.md](AIRGAP_LIVING.md)
- [MCP_LOCAL_ATTACH.md](MCP_LOCAL_ATTACH.md)
- [PRODUCT_MCP.md](PRODUCT_MCP.md) (lite bootstrap)
- [LIVING_APPLIANCE.md](LIVING_APPLIANCE.md)
- [CORE_INSIGHT.md](CORE_INSIGHT.md)
- [ADR-0003-one-instance-metabolism.md](ADR-0003-one-instance-metabolism.md)
