---
type: entity
title: Tensor Parallel (-sm graph / -sm tensor)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Tensor Parallel (-sm graph / -sm tensor)

Type: CONCEPT

## From [[drive-research-cache-optimization-with-ai-chaos-theory|drive-research-cache-optimization-with-ai-chaos-theory]] (2026-06-08)
- Implements tensor parallelism at the GGML graph level, distributing compute graph nodes.
- Achieves high GPU utilization through parallelized mathematical reduction across identical models simultaneously.
- Spreads non-quantized KV cache (f16, bf16, or f32) across devices.
- Fails on Mixture of Experts (MoE) architectures.
- Requires --flash-attn auto/off.
