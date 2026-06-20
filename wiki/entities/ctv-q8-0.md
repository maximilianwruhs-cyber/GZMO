---
type: entity
title: --ctv q8_0
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# --ctv q8_0

Type: TOOL

## From [optimizing-nvidia-blackwell-sm120-part1-micro02](/entities/optimizing-nvidia-blackwell-sm120-part1-micro02.md) (2026-06-09)
- Quantizes the Value cache to 8-bit.
- Quantizes the Key and Value cache to 8-bit.
- Moving the KV cache from raw 16-bit floats (f16) to 8-bit (q8_0) drops the KV cache buffer size by 47%.
