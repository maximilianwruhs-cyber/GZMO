---
type: entity
title: cuda::pipeline
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# cuda::pipeline

Type: TOOL

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- Used by Marlin for Double-Buffered Pipelining.
- Overlaps memory latency by prefetching the compressed weight tile for the upcoming iteration (N+1) while Tensor Cores compute on the active tile (N).
