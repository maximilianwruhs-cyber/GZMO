---
type: entity
title: Edge Hardware
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Edge Hardware

Type: CONCEPT

## From [drive-research-advanced-inference-acceleration](/entities/drive-research-advanced-inference-acceleration.md) (2026-06-08)
- Achieves 2x acceleration when models are small enough to fit in VRAM.
- Pipeline including dual KV caches consumes less than 4 GB VRAM for the 0.5B/3B configuration.

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro01](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro01.md) (2026-06-09)
- Where 2x speedup is achieved because models are small enough to fit in VRAM.
- Common consumer GPUs like NVIDIA RTX 4060 with 8GB VRAM are suitable.
