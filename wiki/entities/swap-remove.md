---
type: entity
title: swap_remove
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# swap_remove

Type: TOOL

## From [[drive-research-rust-ecs-cache-optimization-research|drive-research-rust-ecs-cache-optimization-research]] (2026-06-08)
- A data-oriented solution for efficient element deletion in vectors.
- Achieves O(1) efficiency by swapping with the last element and truncating.
- Must be applied to every component vector in lockstep in a multi-vector system.
