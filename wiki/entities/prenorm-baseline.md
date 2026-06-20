---
type: entity
title: PreNorm baseline
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# PreNorm baseline

Type: CONCEPT

## From [ai-research-part1](/entities/ai-research-part1.md) (2026-06-08)
- A dominant paradigm in modern LLMs.
- Its unweighted accumulation causes hidden-state magnitudes to grow as O(L) with depth.
- AttnRes mitigates PreNorm dilution.
- Restores a clean identity path yet introduces unbounded magnitude growth.
- ∥hl∥ grows as O(L), causing each layer's relative contribution to shrink.
- AttnRes avoids its cumulative magnitude growth.
- One of the variants trained for scaling laws.
- Follows L = 1.891× C−0.057.
- Suffers from the PreNorm dilution problem [60, 27].
