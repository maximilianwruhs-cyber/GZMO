---
type: entity
title: cuda::memcpy_async
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# cuda::memcpy_async

Type: TOOL

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- Used by Marlin for Asynchronous Global Memory Loads.
- Bypasses standard synchronous copy instructions.
- Executes non-blocking weight transfers directly from global memory into shared memory.
- Data movement is managed by dedicated hardware.
