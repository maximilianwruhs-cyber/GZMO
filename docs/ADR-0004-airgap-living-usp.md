# ADR-0004 — Airgap living is the USP (one box, full metabolism)

**Status:** Accepted (2026-07-20)  
**Supersedes for brand / roadmap:** co-primary A+C forever ([SPINE_FOCUS.md](./SPINE_FOCUS.md) migration scaffold)  
**Related:** [ADR-0003-one-instance-metabolism.md](./ADR-0003-one-instance-metabolism.md), [AIRGAP_LIVING.md](./AIRGAP_LIVING.md), [MCP_LOCAL_ATTACH.md](./MCP_LOCAL_ATTACH.md), [LIVING_APPLIANCE.md](./LIVING_APPLIANCE.md)

## Context

Spine packaging treated **A** (laptop Memory MCP, sidecars off) and **C** (living appliance on CT101) as co-primary brands. That read as two products and invited drift toward “ecosystem on a webserver + MCP.”

GZMO’s real differentiator is **sovereign overnight memory metabolism** that can run without the public internet: honeypot + verify + promote + dream/spark/distill on hardware the operator owns.

## Decision

1. **USP dream:** full living Keep on **one airgapped box** — local Prime/embed, local Redis/Qdrant/Neo4j, `gzmo-daemon` overnight writer, agents attach via **local MCP** (stdio / `127.0.0.1` only as the brand path).
2. **Living is first-class development.** Roadmap, quality gates, and installer hero copy aim at airgap living.
3. **Lite (ex-A) is bootstrap / attach-only** — `~/.gzmo` + `gzmo mcp-serve` without overnight writer. Not a peer product roadmap. Exists so day-zero and attach-only clients do not invent a second overnight writer ([ADR-0003](./ADR-0003-one-instance-metabolism.md)).
4. **CT101 is a reference ops deployment** of living, not the cloud brand and not the only allowed living host.
5. **Reject public multi-tenant webserver SKU** as the product. Native MCP-over-HTTP for strangers is out of brand scope; lab HTTP/SSE stays GZMO-next only.
6. **One writer absolute** — never laptop overnight + CT101 overnight at once.
7. **Airgap honesty** — core recall/distill/dream path must not require OpenRouter or public net; cloud LLM is explicit opt-in only.

## Consequences

- Update spine language from “co-primary A+C” → one SKU with **living** vs **lite** profiles ([SPINE_FOCUS.md](./SPINE_FOCUS.md)).
- Continuous quality bar is `scripts/keep-quality-gate.sh` on the living box (not product-MCP GREEN alone).
- Installer narrative: airgap living is the hero; lite is fallback copy ([AIRGAP_LIVING.md](./AIRGAP_LIVING.md)).
- Unpark surfaces attach as local MCP clients after keep-quality soaks GREEN.
