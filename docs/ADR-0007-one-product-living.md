# ADR-0007 — One product: living Keep (no lite SKU)

**Historical status:** Accepted (2026-08-16)
**Decision status:** Superseded
**Implementation status:** Implemented
**Superseded by:** [ADR-0011](./ADR-0011-self-developing-living-database.md) (one-product invariant retained)
**Related:** [ADR-0003](./ADR-0003-one-instance-metabolism.md), [ADR-0004](./ADR-0004-airgap-living-usp.md), [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md), [ADR-0006](./ADR-0006-owner-control-plane.md), [AIRGAP_LIVING.md](./AIRGAP_LIVING.md)

## Context

ADR-0004 kept **lite** (`~/.gzmo`, FTS-only, no overnight) as bootstrap so day-zero clients would not invent a second writer. Spine docs then taught two profiles as if they were two things a customer might want.

Privacy-sensitive customers generate large private corpora. They will look at AnythingLLM / Mem0 / Open WebUI / ChatGPT memory and want **that loop, offline**, with the cable out. A thin SQLite MCP is not that product. Shipping it as “GZMO” trains them to compare us as worse RAG.

Writer vs client still matters. Two *products* does not.

## Decision

1. **One product.** GZMO is the living Keep on one box: local Prime/embed, Redis/Qdrant/Neo4j, hybrid search, folder corpus searchable this sitting, overnight distill/verify/promote, local MCP. That is the install, the demo, and the roadmap.

2. **No lite SKU.** `gzmo init` → `~/.gzmo` without overnight/sidecars/embed is an **incomplete install** or a **telescope/client scratch**, not a brand, not a happy path, not “day-zero GZMO.” Do not market FTS-only Memory MCP as the product.

3. **Writer vs client (ops, not SKU).** One overnight writer per vault ([ADR-0003](./ADR-0003-one-instance-metabolism.md), [ADR-0006](./ADR-0006-owner-control-plane.md)). Cursor / Pi / OpenClaw **attach** to that writer (`gzmo-living`). They do not get a second product brain. The telescope workstation does not run `gzmo serve` while another host holds the living claim.

4. **Degraded living, said out loud.** Sidecars or embedder down is a degraded Keep — fail closed or announce the miss. It is not rebranded as lite.

5. **Customer loop is table stakes, offline.** Same sitting: drop folder → search/chat with citations to the file; hybrid (FTS + vectors) when a local embedder is up; session memory; forget that forgets vault + Qdrant. Overnight: distill → honeypot. Search must label a **corpus passage** vs a **promoted fact**. Cloud embed/chat/public MCP/Telegram-as-brain are leaks, not parity.

6. **Layer A retained.** One writer; airgap honesty (core path needs no public net); no multi-tenant MCP webserver SKU; cloud LLM opt-in only.

## Supersedes

| Was | Now |
|-----|-----|
| ADR-0004 §3 “Lite is bootstrap / attach-only” as brand | Lite is not a product. Attach is a **client** of living. |
| ADR-0005 Layer A §4 “Lite is bootstrap” | Dropped from Layer A. Incomplete `~/.gzmo` is not a peer roadmap *and* not the stranger hero. |
| Spine “A = product MCP, C = living” as co-primary | One SKU: living. A/C is migration vocabulary only. |
| README / `PRODUCT_MCP.md` as stranger product | Hero is [AIRGAP_LIVING.md](./AIRGAP_LIVING.md). `PRODUCT_MCP.md` is client-attach / historical installer notes. |

## Consequences

- Hero install is `install-living-airgap.sh` (or equivalent appliance). `install-gzmo.sh` → `~/.gzmo` must not be described as a complete GZMO.
- Quality bar for the product is `keep-quality-gate.sh` on the living host. `product-readiness-gate.sh` is attach/client smoke, not a second product GREEN.
- Brand MCP label is **`gzmo-living`**. `gzmo-memory` on `~/.gzmo` is legacy.
- Roadmap weight: corpus + hybrid + default local embed on the living box — not another paper joint that only the operator feels.
- Code paths under `~/.gzmo` and `GZMO_PRODUCT=1` may remain for telescope/tests; they are not the customer contract.

## Non-goals

- Two concurrent overnight writers “so the laptop has GZMO too.”
- Pointing a laptop MCP at the living vault as a *second writer*.
- Public HTTP MCP, hosted embeddings, or Telegram as the memory store.
