---
type: entity
title: Unified Memory Architecture (UMA)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Unified Memory Architecture (UMA)

Type: CONCEPT

## From [drive-research-llamacpp-gpu-memory-reporting-bug](/entities/drive-research-llamacpp-gpu-memory-reporting-bug.md) (2026-06-08)
- System RAM and GPU memory share a single physical pool.
- Detected by checking prop.integrated > 0.
- Introduced UMA-aware memory detection in PR #17368.
