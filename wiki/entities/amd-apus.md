---
type: entity
title: AMD APUs
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# AMD APUs

Type: SYSTEM

## From [drive-research-llamacpp-gpu-memory-reporting-bug](/entities/drive-research-llamacpp-gpu-memory-reporting-bug.md) (2026-06-08)
- Systems with Unified Memory Architecture.
- UMA detection logic introduced a performance regression on AMD APUs.
- Allocate GPU memory via the kernel's Translation Table Manager (TTM).
