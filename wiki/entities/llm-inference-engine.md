---
type: entity
title: LLM inference engine
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# LLM inference engine

Type: TOOL

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- When the LLM inference engine is run as a background service managed by systemd, standard PAM settings do not apply.
- To allow memory locking, the service unit must include LimitMEMLOCK=infinity or be run under a security context that grants CapabilityBoundingSet=CAP_IPC_LOCK.
