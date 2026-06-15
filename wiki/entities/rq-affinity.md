---
type: entity
title: rq_affinity
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# rq_affinity

Type: CONCEPT

## From [[drive-research-ubuntu-extreme-hardware-tuning-micro02|drive-research-ubuntu-extreme-hardware-tuning-micro02]] (2026-06-09)
- Block layer setting for interrupt handling affinity.
- Setting to 2 forces I/O completion interrupts on the same CPU core.
- Maximizes L1/L2 cache locality.
