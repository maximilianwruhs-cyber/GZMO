---
type: entity
title: MediaTek MT7925
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# MediaTek MT7925

Type: HARDWARE

## From [[drive-research-analyze-the-pdf-to-create-a-step-by-step-guide-of|drive-research-analyze-the-pdf-to-create-a-step-by-step-guide-of]] (2026-06-08)
- Wi-Fi 7 module.
- ASPM patches and power offsets applied.

## From [[drive-research-ubuntu-extreme-hardware-tuning-micro02|drive-research-ubuntu-extreme-hardware-tuning-micro02]] (2026-06-09)
- Kernel module for MediaTek wireless adapter.
- Configuration option 'disable_aspm=1' is used to disable PCIe ASPM.
- MediaTek wireless adapter.
- PCIe ASPM can be disabled to resolve kernel panics.
- Configuration file for the mt7925 wireless driver.
- Contains options to disable PCIe ASPM.
- Wi-Fi 7 wireless adapter.
- Can experience kernel panics, mutex deadlocks, and connection dropouts under Linux.
- Stability issues often triggered by PCIe Active State Power Management (ASPM) and driver-level power-saving features.
