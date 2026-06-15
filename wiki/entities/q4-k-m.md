---
type: entity
title: Q4_K_M
created: 2026-06-08
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Q4_K_M

Type: CONCEPT

## From [[drive-research-agentic-workflows-fastest-best-models|drive-research-agentic-workflows-fastest-best-models]] (2026-06-08)
- A 4-bit quantization format with medium k-means clustering.
- Provides an optimal equilibrium for agentic workflows.
- Preserves up to 98.9% of unquantized baseline accuracy on reasoning tasks.
- Occupies approximately 15GB to 18GB of VRAM for a 27B model.
- A non-uniform quantization protocol.
- Used to compress weights.
- Helps fit models into VRAM.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro05|drive-research-linux-gaming-and-ai-build-guide-micro05]] (2026-06-09)
- Modern quantization technique.
- Compresses model weights to 4-bit precision.
- Reduces VRAM requirements by approximately 75% compared to FP16.
- Maintains excellent output quality.

## From [[phantom-drive-autonomous-llm-deployment-architect-micro02|phantom-drive-autonomous-llm-deployment-architect-micro02]] (2026-06-10)
- A 4-bit, K-Quant, Medium quantization format.
- The mandatory standard for the Core Zoo to guarantee execution survival on 8GB systems.
