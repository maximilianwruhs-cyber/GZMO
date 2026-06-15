---
type: entity
title: HybridNorm
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# HybridNorm

Type: CONCEPT

## From [[ai-research-part6-micro02|ai-research-part6-micro02]] (2026-06-09)
- Applying LN after Attention residual and normalizing the input of every block.
- Represents a highly competitive variant within the Post-Norm family.
- Outperforms Pre-Norm under conservative learning rates.
- Exhibits training divergence at a learning rate of 1x10^-3.

## From [[ai-research-part6-micro03|ai-research-part6-micro03]] (2026-06-09)
- A Post-Norm variant.
- Provides stability and improves upon its individual components.
- Incorporating it as a sub-stream yields significantly better results.
- Normalized input is inherent.
- Exhibits extreme instability under high learning rates.

## From [[ai-research-part6-micro04|ai-research-part6-micro04]] (2026-06-09)
- Towards stable and efficient transformer training via hybrid normalization
- Published in arXiv preprint arXiv:2503.04598
