---
type: entity
title: nvidia-persistenced.service
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# nvidia-persistenced.service

Type: SYSTEM

## From [drive-research-ultimate-linux-workstation-tuning-blueprint](/entities/drive-research-ultimate-linux-workstation-tuning-blueprint.md) (2026-06-08)
- Required by the rtx5090-nvoc.service.
- Ensures the NVIDIA driver is persistent.
- Part of the NVIDIA driver stack.

## From [drive-research-ubuntu-extreme-hardware-tuning-micro02](/entities/drive-research-ubuntu-extreme-hardware-tuning-micro02.md) (2026-06-09)
- NVIDIA persistence daemon service.
- gpu-oc.service depends on this service.
- Driver for NVIDIA GPUs.
- Configuration option 'NVreg_EnableGpuFirmware=0' is used to disable GSP firmware.
- NVIDIA graphics card.
- GSP firmware can be disabled to prevent power locking and bus dropout bugs.
