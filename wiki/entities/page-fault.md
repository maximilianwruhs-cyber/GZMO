---
type: entity
title: page fault
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# page fault

Type: CONCEPT

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- Occurs when the inference engine accesses swapped-out pages.
- Introduces a millisecond-scale delay while fetching data back from disk.
- Severely degrades token-generation throughput.
