---
type: entity
title: Activation-aware Weight Quantization (AWQ)
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Activation-aware Weight Quantization (AWQ)

Type: TOOL

## From [drive-research-agentic-workflows-fastest-best-models](/entities/drive-research-agentic-workflows-fastest-best-models.md) (2026-06-08)
- An advanced non-uniform quantization technique.
- Required to preserve agentic fidelity.
- Q4_K_M variant occupies approximately 15GB to 18GB of VRAM for a 27B model.

## From [drive-research-32gb-vram-ai-reasoning-models-micro01](/entities/drive-research-32gb-vram-ai-reasoning-models-micro01.md) (2026-06-09)
- Monitors activation distributions during calibration
- Preserves salient, outlier weights in higher precision
- Reduces VRAM requirements by 75% and lifts generation throughput by up to 3.1x compared to FP16
