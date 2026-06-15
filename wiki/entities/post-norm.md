---
type: entity
title: Post-Norm
created: 2026-06-09
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# Post-Norm

Type: CONCEPT

## From [[ai-research-part6-micro01|ai-research-part6-micro01]] (2026-06-09)
- A paradigm with superior potential but unstable.
- Applies LN after the residual addition.
- Recognized for its higher performance upper bound compared to Pre-Norm.

## From [[ai-research-part6-micro02|ai-research-part6-micro02]] (2026-06-09)
- Gradient dynamics are structurally replicated in the bottom-right block of the transition matrix.
- Applying LN after each residual addition.
- Inherent instability becomes increasingly pronounced as learning rate increases.
- Operates in distinct optimum learning rate regimes.

## From [[ai-research-part6-micro03|ai-research-part6-micro03]] (2026-06-09)
- Exhibits extreme instability under high learning rates.
- Severe gradient explosions observed.
- Oscillations typically lead to irreversible training divergence.
- HybridNorm is a Post-Norm variant.

## From [[ai-research-part6-micro04|ai-research-part6-micro04]] (2026-06-09)
- One of the paradigms reconciled by Hyper-Connections and SiameseNorm
