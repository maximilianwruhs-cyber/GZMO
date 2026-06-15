---
type: entity
title: Quantization Techniques
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Quantization Techniques

Type: CONCEPT

## From [[drive-research-agentic-workflows-fastest-best-models|drive-research-agentic-workflows-fastest-best-models]] (2026-06-08)
- Weight quantization is the first step to fitting models into VRAM.
- 8-bit quantization (INT8 or FP8) reduces weight memory footprint by 50%.
- 4-bit quantization is mandatory for certain model classes on 32GB hardware.
- Non-uniform quantization techniques (AWQ, GPTQ, K-Quants) are required for agentic fidelity.
- KV cache quantization (e.g., FP8) is critical for long contexts.
