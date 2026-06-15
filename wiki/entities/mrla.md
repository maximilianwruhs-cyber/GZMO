---
type: entity
title: MRLA
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# MRLA

Type: SYSTEM

## From [[ai-research-part1|ai-research-part1]] (2026-06-08)
- A residual update mechanism.
- Uses a dynamic weight.
- Can access [h1, ..., hl-1] as sources.
- Applies element-wise sigmoid gating over all previous layers.
- Its separable query-key product is closer to linear attention than softmax-based retrieval.
- Corresponds to a structure underlying the MRLA-GLA correspondence.
