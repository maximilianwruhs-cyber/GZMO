---
type: entity
title: GemmGrouped::maximum_active_blocks()
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# GemmGrouped::maximum_active_blocks()

Type: SYSTEM

## From [drive-research-flashinfer-moe-fp4-jit-error](/entities/drive-research-flashinfer-moe-fp4-jit-error.md) (2026-06-08)
- Function used to calculate maximum active blocks
- Returns 0 for several Sm89 tile shapes on Blackwell
- Triggers an occupancy-zero assertion in CUTLASS
