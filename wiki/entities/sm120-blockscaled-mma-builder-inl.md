---
type: entity
title: sm120_blockscaled_mma_builder.inl
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# sm120_blockscaled_mma_builder.inl

Type: TOOL

## From [[drive-research-what-else-can-directly-be-aligned-with-our-common|drive-research-what-else-can-directly-be-aligned-with-our-common]] (2026-06-08)
- Can be manually patched to expose valid K=64 tile templates.

## From [[drive-research-marlin-baseline-for-early-deployments-micro02|drive-research-marlin-baseline-for-early-deployments-micro02]] (2026-06-09)
- Has a 99 KiB shared memory constraint.
- Affected by TMA configuration errors and CuTe DSL descriptor mismatches.
- Requires specific patches for native FP4 execution.
- File that was patched.
- Handles K=64 scale layout constraints.
