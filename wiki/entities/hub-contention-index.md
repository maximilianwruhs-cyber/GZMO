---
type: entity
title: "Hub Contention Index"
created: "2026-06-26"
updated: "2026-06-26"
status: active
tags:
  - research
  - thema_009
  - recall
  - ranking
---

# Hub Contention Index

A per-entity degree cache derived from Neo4j, used to down-weight high-degree hub facts in GZMO's RRF recall. Implements (without superposition math) the paper's finding that high-contention facts are intrinsically harder to retrieve.

## Mechanism

- `scripts/hub-contention-index.py` computes Neo4j degree for entity names referenced in honeypot content.
- Cache: `data/hub-contention-cache.json` — `{entity: {degree, contention_tier: low|med|high}}`.
- In `vault.rs` RRF fusion, facts matching a `high`-contention entity get `score *= hub_contention_penalty` (default 0.85, configurable in `gzmo.toml [recall]`).
- Penalty disabled when the query explicitly names the hub entity (atomic operator lookup, not composition).

## Related

- [Hop-2 Atomic Difficulty](/entities/hop-2-atomic-difficulty.md)
- [Verified Chain Recall](/entities/verified-chain-recall.md)
