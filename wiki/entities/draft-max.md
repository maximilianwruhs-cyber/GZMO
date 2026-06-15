---
type: entity
title: --draft-max
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# --draft-max

Type: TOOL

## From [[optimizing-nvidia-blackwell-sm120-part1-micro02|optimizing-nvidia-blackwell-sm120-part1-micro02]] (2026-06-09)
- Sets the number of tokens to draft per iteration.
- Larger values seem desirable, but an excessively high draft length wastes compute if the target model's acceptance rate collapses deeply into the sequence.
