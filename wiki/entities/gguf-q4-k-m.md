---
type: entity
title: GGUF Q4_K_M
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---





# GGUF Q4_K_M

Type: CONCEPT

## From [[architectural-blueprints-for-sovereign-frankenmoe-part1|architectural-blueprints-for-sovereign-frankenmoe-part1]] (2026-06-08)
- It is a quantization format.
- It has 4.5 bits per weight (bpw).
- A 3x7B FrankenMoE footprint is approximately 10.2 GB with this format.

## From [[architectures-and-optimizations-for-speculative-de-micro04|architectures-and-optimizations-for-speculative-de-micro04]] (2026-06-09)
- A highly efficient quantization format.
- Used for Qwen2.5-3B-Instruct, requiring approximately 2.1 GB of memory.

## From [[drive-research-32gb-vram-ai-reasoning-models-micro03|drive-research-32gb-vram-ai-reasoning-models-micro03]] (2026-06-09)
- A 4-bit quantization regime.
- Mandatory for deploying 30B-class dense models with context windows exceeding 32,000 tokens on a 32 GB GPU.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro04|the-architecture-of-speculative-decoding-and-infer-part1-micro04]] (2026-06-09)
- A type of 4-bit quantization regime.
