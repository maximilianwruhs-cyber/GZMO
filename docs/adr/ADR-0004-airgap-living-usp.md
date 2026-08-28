# ADR-0004 — Airgap living is the USP (one box, full metabolism)

**Status:** Accepted (2026-07-20); process amended by [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md) (2026-07-21); **lite-as-bootstrap brand superseded by [ADR-0007](./ADR-0007-one-product-living.md) (2026-08-16)**  
**Related:** [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md), [ADR-0003](./ADR-0003-one-instance-metabolism.md), [ADR-0007](./ADR-0007-one-product-living.md), [AIRGAP_LIVING.md](../AIRGAP_LIVING.md), [CONTINUOUS_UPGRADE.md](../CONTINUOUS_UPGRADE.md)

## Context

Spine packaging treated laptop Memory MCP and living appliance as co-primary brands. GZMO’s differentiator is **sovereign overnight memory metabolism** on hardware the operator owns.

## Decision (invariants — still binding)

1. **USP dream:** full living Keep on **one airgapped box** — local Prime/embed, local Redis/Qdrant/Neo4j, overnight writer, agents attach via **local MCP** (stdio / `127.0.0.1`).
2. **Living is first-class development** — roadmap and quality gates aim at airgap living ([CONTINUOUS_UPGRADE.md](../CONTINUOUS_UPGRADE.md)).
3. **~~Lite is bootstrap / attach-only~~** — **superseded by [ADR-0007](./ADR-0007-one-product-living.md):** there is no lite SKU. Clients attach to the living writer; `~/.gzmo` is incomplete install / telescope scratch, not a product.
4. **Reject public multi-tenant webserver SKU** — native MCP-over-HTTP for strangers is out of brand scope.
5. **One writer absolute** — never two overnight writers at once (host may move under ADR-0005 mutex).
6. **Airgap honesty** — core recall/distill/dream path must not require OpenRouter or public net; cloud LLM is explicit opt-in only.

## Amended (see ADR-0005)

| Was | Now |
|-----|-----|
| CT101 is *the* living deployment story | CT101 is a **reference** ops deployment; workstation/appliance may claim living under mutex |
| Unpark only after soak GREEN | Soak gates **theater Unpark**; **kernel craft + beat-gate promote** proceed under the flywheel |
| Implied freeze on topology | Topology is Layer B (amendable); USP invariants are Layer A |

## Consequences

- Spine language: **one SKU — living Keep** ([ADR-0007](./ADR-0007-one-product-living.md)). Writer vs client is ops, not a second brand.
- Continuous quality bar remains `keep-quality-gate.sh` on the **current** living host.
- Installer narrative: airgap living is the only hero; `~/.gzmo` FTS MCP is not a complete GZMO.
- Continuous upgrade flywheel ([ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md)) is how living gets better every week — ADRs must not block it.
