---
type: entity
title: Linux Capabilities
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---




# Linux Capabilities

Type: CONCEPT

## From [[drive-research-architecting-a-linux-task-manager-design-principl|drive-research-architecting-a-linux-task-manager-design-principl]] (2026-06-08)
- Partition the root user's power into discrete units.
- Granting specific capabilities to the task manager binary allows it to bypass standard permission checks.

## From [[gzmo-soul-merged-new-part1|gzmo-soul-merged-new-part1]] (2026-06-09)
- Granular security model that partitions root user privileges into discrete functional units

## From [[drive-research-automating-linux-hardware-detection-micro03|drive-research-automating-linux-hardware-detection-micro03]] (2026-06-09)
- Mitigate the risk of granting full root access.
- Partition the monolithic root privilege into granular operational units.
- Managed through a complex system of sets evaluated by the kernel.
- CAP_SYS_ADMIN is a catch-all for system administration.
- CAP_AUDIT_READ is required for security scanners.
- CAP_BPF is used for privileged Berkeley Packet Filter operations.
- CAP_BLOCK_SUSPEND prevents sleep states during hardware scans.
- CAP_CHECKPOINT_RESTORE reads PID mapping files.
- Hardware discovery containers are configured to drop ALL capabilities by default.
- Strict adherence to Linux capability sets is demanded.
