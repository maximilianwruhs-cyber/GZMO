---
type: entity
title: Triton Kernel Optimization
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Triton Kernel Optimization

Type: CONCEPT

## From [[drive-research-speicherbandbreiten-engpass-memory-wall|drive-research-speicherbandbreiten-engpass-memory-wall]] (2026-06-08)
- Used in vLLM for high-throughput serving.
- Pre-rotates the Query (Q) vector with the inverse of the PolarQuant rotation matrix.
- Computes attention scores by directly gathering centroid values via table lookups.
