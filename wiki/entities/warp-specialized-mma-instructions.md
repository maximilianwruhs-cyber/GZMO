---
type: entity
title: Warp-Specialized MMA instructions
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Warp-Specialized MMA instructions

Type: CONCEPT

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01.md) (2026-06-09)
- Hardware features that consume data directly from shared memory.
- Enforce strict, low-level alignment boundaries on physical memory addresses.
- Require target shared memory addresses to maintain a minimum 16-byte alignment for instructions like ldmatrix.sync.aligned.
