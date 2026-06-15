---
type: entity
title: INT4
created: 2026-06-09
updated: 2026-06-09
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---






# INT4

Type: CONCEPT

## From [[drive-research-32gb-vram-ai-reasoning-models-micro01|drive-research-32gb-vram-ai-reasoning-models-micro01]] (2026-06-09)
- 4-bit quantization
- Each parameter requires 0.5 bytes
- Can be used for KV cache quantization

## From [[drive-research-32gb-vram-ai-reasoning-models-micro03|drive-research-32gb-vram-ai-reasoning-models-micro03]] (2026-06-09)
- Quantization level for KV cache.
- Structural necessity for pushing context lengths toward 128K or 256K.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro01|drive-research-linux-gaming-and-ai-build-guide-micro01]] (2026-06-09)
- Operations relying on INT4 are accelerated by Tensor Cores.
- INT4 is a numerical format used in AI computations.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro04|optimizing-nvidia-blackwell-sm120-part1-micro04]] (2026-06-09)
- Marlin is capable of executing FP16 x INT4 operations.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro04|the-architecture-of-speculative-decoding-and-infer-part1-micro04]] (2026-06-09)
- Can be used for quantizing the KV cache.
