---
type: entity
title: RLIMIT_MEMLOCK
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# RLIMIT_MEMLOCK

Type: CONCEPT

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- A resource limit designated by the Linux kernel that restricts memory-locking operations.
- Dictates the maximum virtual address space a process can lock into physical RAM.
- Significant confusion arose among developers regarding whether driver-level host memory allocations were bound by this parameter.
- Modern Linux kernels and contemporary CUDA driver runtimes strictly enforce standard user-space resource accounting against this limit.
- If the size of the memory buffer exceeds this limit, the system call fails.
- The default RLIMIT_MEMLOCK on many Linux distributions is highly restrictive.
- The administrative Linux capability CAP_IPC_LOCK is not required if the user's RLIMIT_MEMLOCK resource limits are configured high enough.
- Under modern kernel rules, for unprivileged processes, memory locking is permitted up to the maximum threshold defined by the process's effective RLIMIT_MEMLOCK soft resource limit.
- A standard workstation user can raise their own soft limit up to the ceiling defined by the hard limit.
- Configuration directives must be defined in the PAM limits configuration subsystem to apply these resource limits persistently.
