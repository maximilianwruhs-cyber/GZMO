---
type: entity
title: mHC
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# mHC

Type: SYSTEM

## From [ai-research-part1](/entities/ai-research-part1.md) (2026-06-08)
- A prior residual generalization.
- Has a per-layer memory access cost of 34d reads and 2m2+4m writes (Table 1).
- Improves loss to 1.747 in ablation studies.
- A stabilized variant of Hyper-Connections.
- Its weight Mi→l = β⊤i A×i+1→lαl admits a natural attention interpretation.
- Acts as depth-wise linear attention with matrix-valued states.

## From [ai-research-part7](/entities/ai-research-part7.md) (2026-06-08)
- Stands for Manifold-Constrained Hyper-Connections.
- A general framework that projects the residual connection space of HC onto a specific manifold to restore the identity mapping property.
- Incorporates rigorous infrastructure optimization to ensure efficiency.
- It is a system for which the calculation process of Hpre, Hpost, and Hres is detailed.
- It imposes significant latency when RMSNorm operates on the high-dimensional hidden state.
- It is implemented with n=4 in large-scale models with a marginal training overhead of only 6.7%.
- Significantly enhances propagation stability compared to HC.
- Reduces maximum gain magnitude by three orders of magnitude compared to HC.

## From [ai-research-part6-micro04](/entities/ai-research-part6-micro04.md) (2026-06-09)
- Variant of Hyper-Connections
- Adopts a design that more closely aligns with the Pre-Norm paradigm
- Ablation studies indicate Hres is critical for performance gains
