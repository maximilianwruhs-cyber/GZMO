---
type: entity
title: CapabilityBoundingSet=CAP_IPC_LOCK
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# CapabilityBoundingSet=CAP_IPC_LOCK

Type: CONCEPT

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- Must be granted to a service unit file to allow memory locking for systemd-managed services.
- Grants the CAP_IPC_LOCK capability to a systemd service.
- An administrative Linux capability.
- Not strictly required to utilize multi-gigabyte cudaHostRegister buffers on a single-user workstation if RLIMIT_MEMLOCK is configured sufficiently high.
- Historically, a process was strictly required to hold root privileges or possess CAP_IPC_LOCK to perform any memory-locking operations.
- Under modern kernel rules, no limits are placed on the amount of memory a privileged process (possessing CAP_IPC_LOCK) can lock.
- Required for containerized environments if the container is not launched with --ulimit memlock=-1.
- Can be granted to a service unit file via CapabilityBoundingSet=CAP_IPC_LOCK.
