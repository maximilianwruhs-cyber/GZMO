---
type: entity
title: Infinity Fabric Clock (FCLK)
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Infinity Fabric Clock (FCLK)

Type: HARDWARE

## From [[drive-research-ultimate-linux-workstation-tuning-blueprint|drive-research-ultimate-linux-workstation-tuning-blueprint]] (2026-06-08)
- Optimal latency-reducing frequency is 2167 MHz.
- Maintained at a 1:1 synchronization ratio with MCLK and UCLK.
- Cross-CCD migration across this interconnect is prohibited.

## From [[drive-research-ubuntu-extreme-hardware-tuning-micro01|drive-research-ubuntu-extreme-hardware-tuning-micro01]] (2026-06-09)
- Clock speed that must be synchronized with MCLK and UCLK.
- Should be set manually to its maximum stable frequency for high-speed DDR5.
