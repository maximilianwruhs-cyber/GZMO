---
type: entity
title: Quantization
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Quantization

Type: CONCEPT

## From [drive-research-hidden-mode-technical-analysis-and-configurati](/entities/drive-research-hidden-mode-technical-analysis-and-configurati.md) (2026-06-08)
- Essential for consumer or edge-node deployments due to VRAM utilization.
- Extreme quantization (e.g., 2-bit or 3-bit) often degrades the routing performance of MoEs.
- Formats like GGUF offer various bit-per-weight options.

## From [drive-research-token-efficient-bol-processing-architecture](/entities/drive-research-token-efficient-bol-processing-architecture.md) (2026-06-08)
- Aggressive quantization strategies reduce the memory footprint of models.
- Examples include 4-bit block-wise quantization (Q4_0, Q4_K_M) and 1.58-bit ternary quantizations.
- Ternary models restrict network weights to values of -1, 0, and 1.
