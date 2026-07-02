---
type: entity
title: "Verified Chain Recall (VCR)"
created: "2026-06-26"
updated: "2026-06-26"
status: active
tags:
  - research
  - thema_009
  - recall
  - knowledge-graph
  - compositional
---

# Verified Chain Recall (VCR)

GZMO-native approach to multi-hop/compositional recall, derived as the **positive alternative** to holographic memory (thema_009). Decompose a multi-hop query into per-hop atomic honeypot lookups, verify each intermediate against Neo4j **and** the honeypot, then emit structured chain hints. No bind-unbind algebra, no superposition.

## Why not holographic memory

arXiv:2606.24948 shows HRR/FHRR reach MRR ~0.35 single-hop but **fail at chance** on zero-shot two-hop composition. The bottleneck is retrieval capacity under superposition for high-contention facts, not the cleanup operator. HRR capacity (~50 clean facts in 1024D) is orders of magnitude below GZMO's 22k+ honeypot points. See [Holographic Reduced Representations (HRR)](/entities/holographic-reduced-representations.md).

## Architecture analogue

Mirrors [Continuous Query Decomposition (CQD)](/entities/continuous-query-decomposition-cqd.md) (ICLR 2021): score each atom independently, aggregate, emit explainable intermediates — but using GZMO's explicit graph + honeypot instead of a neural link predictor.

## Implementation in survey_GZMO

| Component | Path |
|-----------|------|
| 2-hop graph stream | `scripts/graph-recall-stream.py --mode=2hop` |
| Compositional probe | `scripts/compositional-recall-probe.py` |
| Hub contention weighting | `scripts/hub-contention-index.py` + `vault.rs` RRF |
| Discovery LINK chain contract | `chain_recall_query` in link-registry |
| Gate | `scripts/ingest-quality/gate-discovery-loop.sh` (atomic vs chain) |

## Related

- [Compositional Recall Capacity](/entities/compositional-recall-capacity.md)
- [Hub Contention Index](/entities/hub-contention-index.md)
- [Spreading Activation](/entities/spreading-activation.md)
- Synthesis: `docs/THEMA_009_COMPOSITIONAL_RECALL_SYNTHESIS.md`
