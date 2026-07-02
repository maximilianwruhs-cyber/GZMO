---
type: entity
title: "Hop-2 Atomic Difficulty"
created: "2026-06-26"
updated: "2026-06-26"
status: draft
tags:
  - research
  - thema_009
  - recall
---

# Hop-2 Atomic Difficulty

thema_009 probe: posing the ground-truth second-hop fact as a standalone atomic query (no composition) already recovers it at only **0.26–0.48×** the model's average atomic accuracy, uniformly across relation fan-out. The bottleneck is upstream of composition — the facts chains depend on are intrinsically harder to retrieve from a superposed memory.

## GZMO translation

High-degree hub entities in GZMO's honeypot/Neo4j are the analogue: dense, heavily-referenced facts that are harder to retrieve precisely even standalone. Mitigated by the [Hub Contention Index](/entities/hub-contention-index.md), which down-weights such facts in RRF unless the query explicitly names them.

## Related

- [Compositional Recall Capacity](/entities/compositional-recall-capacity.md)
- [Hub Contention Index](/entities/hub-contention-index.md)
