---
type: entity
title: Manifold-Constrained Hyper-Connections (mHC)
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Manifold-Constrained Hyper-Connections (mHC)

Type: CONCEPT

## From [[ai-research-part8-micro03|ai-research-part8-micro03]] (2026-06-09)
- Resolves divergence by subjecting residual mapping matrices to strict geometric constraints.
- Forces learnable transition matrices to reside exclusively on the Birkhoff polytope.
- Scales smoothly to 27B parameters.
- Introduces only a 6.7% additional time overhead compared to standard architectures with an expansion rate of n=4.

## From [[ai-research-part8-micro07|ai-research-part8-micro07]] (2026-06-09)
- It utilizes Sinkhorn-Knopp algorithms.
- It projects learnable transition matrices onto the Birkhoff polytope.
- It prevents signal explosions in widened residual streams.
