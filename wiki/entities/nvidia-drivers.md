---
type: entity
title: NVIDIA drivers
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# NVIDIA drivers

Type: TOOL

## From [drive-research-du-hast-gesagt-part1](/entities/drive-research-du-hast-gesagt-part1.md) (2026-06-08)
- Proprietary drivers needed for NVIDIA GPU.
- Can be installed via `ubuntu-drivers autoinstall`.

## From [drive-research-enhancing-local-ai-hypervisor-architecture](/entities/drive-research-enhancing-local-ai-hypervisor-architecture.md) (2026-06-08)
- Must be installed directly on the Proxmox host for shared container access.
- Proprietary Linux driver installer can be downloaded and executed.
- Can be installed within an LXC container using the '--no-kernel-modules' flag.
