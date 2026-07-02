---
type: entity
title: "Compositional Recall Capacity"
created: "2026-06-26"
updated: "2026-06-26"
status: draft
tags:
  - research
  - thema_009
  - recall
  - knowledge-graph
---

# Compositional Recall Capacity

The ability of a memory system to answer multi-hop queries whose relation chains were not observed at training time. Holographic memory fails this zero-shot (arXiv:2606.24948); the failure is a **retrieval-capacity** effect, not a bind-unbind algebra or cleanup problem.

## Mechanism (from thema_009)

- Aggregate metrics (MRR) hide per-fact weakness: a near-miss costs an atomic query a few ranks but costs a compositional query the entire prediction.
- The facts compositional chains depend on are, by construction, the higher-degree facts that suffer most cross-talk under superposition.
- Measurable at a single hop, before any cleanup step acts.

## GZMO relevance

GZMO has no compositional recall eval; `discovery-kb-recall-smoke.sh` only checks atomic non-empty hits (falsely green at 100%). [Verified Chain Recall](/entities/verified-chain-recall.md) adds the missing probe. Closes G12 (eval green ≠ recall green).

## Related

- [Hop-2 Atomic Difficulty](/entities/hop-2-atomic-difficulty.md)
- [Holographic Reduced Representations (HRR)](/entities/holographic-reduced-representations.md)
