---
type: entity
title: Full AttnRes
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Full AttnRes

Type: SYSTEM

## From [ai-research-part1](/entities/ai-research-part1.md) (2026-06-08)
- One of the variants trained for scaling laws.
- Fits L = 1.865 × C−0.057.
- Achieves the lowest loss (1.737) in ablation studies when applying attention over all previous layers.
- Corresponds to N=L in the context of effective rank of M.
- Must access all preceding layer outputs at every layer.
- Memory footprint of cross-layer aggregation grows as O(Ld).
- Introduces only one RMSNorm and one pseudo-query vector wl ∈ Rd per layer.
- Consistently achieves lower loss across the entire compute range compared to the baseline.
- Outperforms mHC while matching its lower memory I/O per layer.
- Replaces fixed, uniform residual accumulation with learned, input-dependent depth-wise attention.
- Its input-dependent M reveals depth-wise attention sinks.
- Acts as depth-wise softmax attention.
