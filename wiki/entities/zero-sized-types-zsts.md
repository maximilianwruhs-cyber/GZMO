---
type: entity
title: Zero-Sized Types (ZSTs)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Zero-Sized Types (ZSTs)

Type: CONCEPT

## From [drive-research-cache-optimization-blueprint](/entities/drive-research-cache-optimization-blueprint.md) (2026-06-08)
- Prevented from growing allocations in CacheAlignedVec<T>.
- Handled by `calculate_layout` function.
