---
type: entity
title: Custom K=64 Tile Templates
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Custom K=64 Tile Templates

Type: CONCEPT

## From [drive-research-what-else-can-directly-be-aligned-with-our-common](/entities/drive-research-what-else-can-directly-be-aligned-with-our-common.md) (2026-06-08)
- Exposed to the autotuner by patching sm120_blockscaled_mma_builder.inl and modifying generate_kernels.py.
- Directly reduces shared memory allocation required for the tile.
- Allows compiled kernels to fit completely within the workstation-specific 99 KiB hardware envelope.
