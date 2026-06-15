---
type: entity
title: CPU-bound sampling bottleneck
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# CPU-bound sampling bottleneck

Type: CONCEPT

## From [[drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02|drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02]] (2026-06-09)
- Performance degradation is caused by a sequential CPU-bound sampling bottleneck.
- Token selection algorithms have historically run sequentially on the CPU.
- Enabling experimental GPU-accelerated sampling bypasses this sampling bottleneck.
