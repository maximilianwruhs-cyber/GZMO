---
type: entity
title: VFIO
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# VFIO

Type: CONCEPT

## From [drive-research-enhancing-local-ai-hypervisor-architecture](/entities/drive-research-enhancing-local-ai-hypervisor-architecture.md) (2026-06-08)
- Isolates the physical graphics processor to a single virtualized guest.
- Requires full physical PCIe passthrough via host-level bindings.
- If bound to virtual machines via VFIO drivers, the host's native kernel would blacklist NVIDIA modules.
