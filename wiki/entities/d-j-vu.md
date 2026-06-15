---
type: entity
title: DéjàVu
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# DéjàVu

Type: TOOL

## From [[drive-research-token-efficient-bol-processing-architecture|drive-research-token-efficient-bol-processing-architecture]] (2026-06-08)
- Provides advanced KV cache streaming libraries.
- Disaggregates prompt processing from token generation.
- Maximizes memory allocation and enables high-throughput streaming by swapping KV cache state per-microbatch between the host CPU and the GPU.
