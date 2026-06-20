---
type: entity
title: OOM killer
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# OOM killer

Type: SYSTEM

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- The kernel's Out-of-Memory (OOM) subsystem is activated when physical RAM is exhausted.
- Calculates a badness score (oom_score) for every active process.
- Targets memory-intensive, non-privileged processes for termination to protect the host kernel from crashing.
- A GGML or llama.cpp process pinning multi-gigabyte weight buffers occupies a massive portion of physical RAM, making it a primary target for the OOM killer.
- The kernel will send a fatal SIGKILL (Signal 9) to the inference application, terminating the model instantly.
- The kernel will send a fatal SIGKILL (Signal 9) to the inference application if it is targeted by the OOM killer.
