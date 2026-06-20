---
type: entity
title: online softmax
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# online softmax

Type: CONCEPT

## From [ai-research-part1](/entities/ai-research-part1.md) (2026-06-08)
- Used by Phase 2 to merge with Phase 1 outputs.
- Its merge is elementwise, admitting kernel fusion with surrounding operations.
- Integrates into the standard TP all-reduce communication path during memory-efficient prefilling.
