---
type: entity
title: EXL2
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# EXL2

Type: CONCEPT

## From [drive-research-32gb-vram-ai-reasoning-models-micro01](/entities/drive-research-32gb-vram-ai-reasoning-models-micro01.md) (2026-06-09)
- An executable file format for quantized models
- Proven empirically superior to GGUF in local inference velocities on 32 GB hardware
- Targets fractional bitrates for granular control

## From [drive-research-32gb-vram-ai-reasoning-models-micro03](/entities/drive-research-32gb-vram-ai-reasoning-models-micro03.md) (2026-06-09)
- A 4-bit quantization regime.
- Mandatory for deploying 30B-class dense models with context windows exceeding 32,000 tokens on a 32 GB GPU.
- Provides the most mathematically efficient utilization of exact VRAM boundaries when targeting ~4.5 to 4.65 bits per weight.
