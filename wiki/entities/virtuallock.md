---
type: entity
title: VirtualLock
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# VirtualLock

Type: TOOL

## From [architectural-strategy-for-stealthy-portable-cli-a](/entities/architectural-strategy-for-stealthy-portable-cli-a.md) (2026-06-08)
- The agent must utilize system-level memory locking APIs—such as VirtualLock on Windows.
- VirtualLock is used to prevent the host operating system from accidentally writing key material to the physical disk via memory paging.

## From [drive-research-architecting-zero-configuration-portable-agents-s-micro03](/entities/drive-research-architecting-zero-configuration-portable-agents-s-micro03.md) (2026-06-09)
- System-level memory locking API on Windows.
- Used to prevent memory paging of key material.
- Prevents memory paging of key material.
