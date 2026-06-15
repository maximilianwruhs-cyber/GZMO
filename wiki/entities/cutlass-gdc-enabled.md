---
type: entity
title: CUTLASS_GDC_ENABLED
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# CUTLASS_GDC_ENABLED

Type: TOOL

## From [[optimizing-nvidia-blackwell-sm120-part1-micro04|optimizing-nvidia-blackwell-sm120-part1-micro04]] (2026-06-09)
- Explicitly adding this compiler flag compiles GDC barriers as actual PTX instructions.
- Output remains corrupted even when used.
