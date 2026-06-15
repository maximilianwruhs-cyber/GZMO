---
type: entity
title: 4-bit quantization
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# 4-bit quantization

Type: CONCEPT

## From [[drive-research-32gb-vram-ai-reasoning-models-micro01|drive-research-32gb-vram-ai-reasoning-models-micro01]] (2026-06-09)
- Mandatory when VRAM is the absolute binding constraint for large models and context windows
- Methodologies exist to minimize intelligence loss
- Sophisticated methodologies exist to minimize intelligence loss

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro04|the-architecture-of-speculative-decoding-and-infer-part1-micro04]] (2026-06-09)
- Strictly mandates deploying 30B-class dense models with context windows exceeding 32,000 tokens on a 32 GB GPU.
- Includes AWQ, EXL2, or GGUF Q4_K_M.
