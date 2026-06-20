---
type: entity
title: /etc/sysctl.d/99-swappiness.conf
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# /etc/sysctl.d/99-swappiness.conf

Type: TOOL

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- The vm.swappiness sysctl parameter determines how aggressively the Linux kernel moves anonymous memory pages to swap storage.
- sysctl can be used to apply swappiness modifications temporarily or persistently.
- A configuration file used to persistently apply swappiness modifications across system restarts.
- The line 'vm.swappiness=10' would be added to this file.
