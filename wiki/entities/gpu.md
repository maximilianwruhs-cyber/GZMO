---
type: entity
title: GPU
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# GPU

Type: SYSTEM

## From [[drive-research-hermes-anthropic-openrouter-cache-investigation|drive-research-hermes-anthropic-openrouter-cache-investigation]] (2026-06-08)
- Graphics Processing Unit.
- Manages KV-Cache in VRAM.
- Distributed execution engines are enabled by frameworks like Ray.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro02|optimizing-nvidia-blackwell-sm120-part1-micro02]] (2026-06-09)
- Physical micro-batch size is the actual first dimension of the tensor matrices deployed into the GPU's memory pool.
- High logical batch size allows the engine to execute massive parallel matrix multiplications during the prompt prefill phase, maximizing the CUDA cores' arithmetic intensity.
- Continuous batching ensures that the GPU pipeline remains fully saturated across all layer computations.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro05|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro05]] (2026-06-09)
- Real-time performance monitoring is facilitated by powerstat and nvidia-smi.
