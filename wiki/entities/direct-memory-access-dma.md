---
type: entity
title: Direct Memory Access (DMA)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Direct Memory Access (DMA)

Type: CONCEPT

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- Standard memory requires the CUDA driver to copy data into an internal, page-locked staging buffer before performing a DMA transfer to the GPU.
- Page-locked memory allows the GPU's onboard DMA engines to copy weights directly across the PCIe bus.
- This bypasses the host CPU entirely and achieves maximum theoretical hardware bandwidth.
