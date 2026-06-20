---
type: entity
title: Key-Value cache
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Key-Value cache

Type: CONCEPT

## From [optimizing-nvidia-blackwell-sm120-part1-micro02](/entities/optimizing-nvidia-blackwell-sm120-part1-micro02.md) (2026-06-09)
- Memory usage scales linearly with context size due to its continuous materialization.
- Flash Attention radically reduces its size by preventing N x N matrix materialization.
- Parallel decoding inherently risks severe KV cache fragmentation.
- System architects must quantize the KV cache itself to preserve stability when context window expands.
