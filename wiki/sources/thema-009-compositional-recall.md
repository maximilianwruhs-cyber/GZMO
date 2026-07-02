---
type: source
title: thema_009-compositional-recall
created: 2026-06-26
updated: 2026-06-26
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# thema_009-compositional-recall

Ingested source summary (2026-06-26). Negative-result study of holographic memory for zero-shot KG composition, integrated as a Verified Chain Recall diagnostic for GZMO.

## Entities
- [Holographic Reduced Representations (HRR)](/entities/holographic-reduced-representations.md) (CONCEPT)
- [Compositional Recall Capacity](/entities/compositional-recall-capacity.md) (CONCEPT)
- [Hop-2 Atomic Difficulty](/entities/hop-2-atomic-difficulty.md) (CONCEPT)
- [Hub Contention Index](/entities/hub-contention-index.md) (CONCEPT)
- [Verified Chain Recall](/entities/verified-chain-recall.md) (SYSTEM)
- [Continuous Query Decomposition (CQD)](/entities/continuous-query-decomposition-cqd.md) (SYSTEM)

## Relations (manual links, 2026-06-26)

LINK: [Holographic Reduced Representations (HRR)](/entities/holographic-reduced-representations.md) —fails-at→ [Compositional Recall Capacity](/entities/compositional-recall-capacity.md) | WHY: zero-shot two-hop composition at chance across all cleanup temperatures (arXiv:2606.24948)

LINK: [Compositional Recall Capacity](/entities/compositional-recall-capacity.md) —limited-by→ [Hop-2 Atomic Difficulty](/entities/hop-2-atomic-difficulty.md) | WHY: hop-2 facts retrieved at 0.26–0.48× atomic baseline even without composition

LINK: [Verified Chain Recall](/entities/verified-chain-recall.md) —alternative-to→ [Holographic Reduced Representations (HRR)](/entities/holographic-reduced-representations.md) | WHY: explicit per-hop atomic lookups + verified Neo4j walk; no superposition

LINK: [Verified Chain Recall](/entities/verified-chain-recall.md) —analogue-of→ [Continuous Query Decomposition (CQD)](/entities/continuous-query-decomposition-cqd.md) | WHY: decompose query into atomic predictions, aggregate; explainable intermediates

LINK: [Hub Contention Index](/entities/hub-contention-index.md) —mitigates→ [Hop-2 Atomic Difficulty](/entities/hop-2-atomic-difficulty.md) | WHY: down-weights high-degree hub facts in RRF that are structurally hard to retrieve

LINK: [Verified Chain Recall](/entities/verified-chain-recall.md) —closes→ [G12 gap](/entities/g12-eval-green-recall-green.md) | WHY: compositional probe separates eval-green from recall-green
