---
type: entity
title: Pre-Norm
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Pre-Norm

Type: CONCEPT

## From [ai-research-part6-micro02](/entities/ai-research-part6-micro02.md) (2026-06-09)
- Gradient dynamics are structurally replicated in the bottom-right block of the transition matrix.
- Gradient flow is modulated by the normalization Jacobian JLN j in the top-left block.
- Applying LN before each residual addition.
- Maintains training stability and consistently increases its average downstream score.
- Operates in distinct optimum learning rate regimes.

## From [ai-research-part6-micro03](/entities/ai-research-part6-micro03.md) (2026-06-09)
- Provides optimization stability.
- Standard architecture.
- Gradient norms consistently remain below 0.5 after warm-up phase.
- Can exhibit degraded depth utilization in very deep Transformers.
- Unnormalized residual stream can increase in magnitude with depth.
