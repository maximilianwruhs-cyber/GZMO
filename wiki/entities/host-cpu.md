---
type: entity
title: host CPU
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# host CPU

Type: SYSTEM

## From [[drive-research-cuda-graph-capture-failure-workarounds-micro03|drive-research-cuda-graph-capture-failure-workarounds-micro03]] (2026-06-09)
- Must rebuild the entire execution graph at every sequential token decoding step without graph reuse.
- Rebuild cycle introduces significant CPU overhead on dual Tesla V100 systems.
