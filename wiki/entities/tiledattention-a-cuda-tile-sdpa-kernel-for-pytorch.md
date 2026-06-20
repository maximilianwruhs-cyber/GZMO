---
type: entity
title: 'TiledAttention: a CUDA Tile SDPA Kernel for PyTorch'
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# TiledAttention: a CUDA Tile SDPA Kernel for PyTorch

Type: BOOK

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro02](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro02.md) (2026-06-09)
- This is a research paper or publication.
- It is related to TiledAttention and PyTorch.
- cuTile was introduced experimentally in CUDA 13.1 and stabilized in 13.2.
- CUDA graph capture pressure can trigger kernel power failures.
- CUDA Toolkit 13.1 Update 2 release notes are mentioned.
- Implements a blockwise online-softmax formulation.
- Streams keys and values through shared memory.
- Uses a JIT-compiled tiled attention formulation.
