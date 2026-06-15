---
type: entity
title: NVMe issues
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# NVMe issues

Type: HARDWARE

## From [[drive-research-ultimate-linux-workstation-tuning-blueprint|drive-research-ultimate-linux-workstation-tuning-blueprint]] (2026-06-08)
- Controller benefits from disabling APST.
- Requires specific I/O ring parameters.
- Utilizes io_uring interface for optimization.
- resurfacing with PVE kernel upgrade
- topic of a Proxmox forum thread
