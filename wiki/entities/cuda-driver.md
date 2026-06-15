---
type: entity
title: CUDA driver
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# CUDA driver

Type: SYSTEM

## From [[drive-research-rust-ecs-cache-optimization-research|drive-research-rust-ecs-cache-optimization-research]] (2026-06-08)
- Manages GPU contexts and access.
- Allows only one active CUDA context per device in EXCLUSIVE_PROCESS mode.
- Denies access to external processes attempting to initialize a context.
