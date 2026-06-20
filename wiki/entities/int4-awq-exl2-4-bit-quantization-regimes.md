---
type: entity
title: INT4 / AWQ / EXL2 (4-bit quantization regimes)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# INT4 / AWQ / EXL2 (4-bit quantization regimes)

Type: CONCEPT

## From [drive-research-32gb-vram-ai-reasoning-models-micro01](/entities/drive-research-32gb-vram-ai-reasoning-models-micro01.md) (2026-06-09)
- An executable file format for quantized models
- Empirically superior to GGUF in local inference velocities on 32 GB hardware
- Allows targeting fractional bitrates (e.g., 4.65 bpw, 5.0 bpw)
- Each parameter is compressed to 0.5 bytes
- A 32B model occupies 16-18 GB of VRAM
- Secures 14-16 GB of headroom for KV cache and runtime
- Each parameter requires 0.5 bytes
- Leaves 14-16 GB headroom for KV cache and runtime
