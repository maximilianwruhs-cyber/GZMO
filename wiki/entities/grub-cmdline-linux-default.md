---
type: entity
title: GRUB_CMDLINE_LINUX_DEFAULT
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# GRUB_CMDLINE_LINUX_DEFAULT

Type: CONCEPT

## From [drive-research-ubuntu-extreme-hardware-tuning-micro02](/entities/drive-research-ubuntu-extreme-hardware-tuning-micro02.md) (2026-06-09)
- Kernel boot command line parameter.
- Used to pass system-wide performance parameters on boot.
- Includes parameters like 'amd_iommu=on', 'iommu=pt', 'amd_pstate=active', 'nvme_core.default_ps_max_latency_us=0', 'rcutree.enable_rcu_lazy=1', 'audit=0'.
- Virtual memory engine organizes system memory into 4096-byte (4 KiB) pages.
- Standard kernel block-layer configurations must be optimized for high-performance NVMe storage.
- Default TCP stack is configured conservatively.
