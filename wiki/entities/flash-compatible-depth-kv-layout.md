---
type: entity
title: Flash-compatible depth KV layout
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Flash-compatible depth KV layout

Type: CONCEPT

## From [ai-research-part5](/entities/ai-research-part5.md) (2026-06-08)
- Flattens the depth cache along a single axis.
- Turns depth lookup into contiguous block reads.
- Compatible with FlashAttention-style kernels.
