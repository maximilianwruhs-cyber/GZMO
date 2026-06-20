---
type: entity
title: micro-batch sizing
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# micro-batch sizing

Type: CONCEPT

## From [drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02](/entities/drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02.md) (2026-06-09)
- On systems utilizing unified memory pools, maximize micro-batch sizing.
- This reduces memory thrashing.
- Avoid configuring redundant CPU-side caching to prevent system memory fragmentation.
- Harmonize batch sizing to hardware capacity.
- Configure --batch-size to match the maximum expected input prompt size.
- Maximize --ubatch-size within hardware limits to ensure weights are streamed efficiently.
