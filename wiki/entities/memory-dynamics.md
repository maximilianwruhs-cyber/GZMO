---
type: entity
title: Memory Dynamics
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Memory Dynamics

Type: CONCEPT

## From [drive-research-agentic-workflows-fastest-best-models](/entities/drive-research-agentic-workflows-fastest-best-models.md) (2026-06-08)
- Total VRAM during inference is sum of model weights, KV cache, and activation memory.
- KV cache is the dominant memory consumer as sequence length increases.
- Exceeding VRAM limits causes massive bandwidth bottlenecks.
