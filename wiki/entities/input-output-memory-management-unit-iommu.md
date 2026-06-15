---
type: entity
title: Input-Output Memory Management Unit (IOMMU)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Input-Output Memory Management Unit (IOMMU)

Type: CONCEPT

## From [[drive-research-ultimate-linux-workstation-tuning-blueprint|drive-research-ultimate-linux-workstation-tuning-blueprint]] (2026-06-08)
- Pass-through is enabled via iommu=pt.
- Reduces translation overhead for Direct Memory Access (DMA) mapping.
- Required for unthrottled GPU PCIe 5.0 communication.
