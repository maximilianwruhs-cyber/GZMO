---
type: entity
title: Netlink Proc Connector (cn_proc)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Netlink Proc Connector (cn_proc)

Type: SYSTEM

## From [drive-research-architecting-a-linux-task-manager-design-principl](/entities/drive-research-architecting-a-linux-task-manager-design-principl.md) (2026-06-08)
- An asynchronous multicast socket interface.
- Introduced in kernel version 2.6.15.
- Pushes real-time process events (like fork(), exec(), exit()) directly from the kernel to user space.
- Eliminates the need for brute-force directory traversal.
- Binding to this connector generally requires root privileges.
