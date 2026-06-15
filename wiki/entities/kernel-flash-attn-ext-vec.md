---
type: entity
title: kernel_flash_attn_ext_vec
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# kernel_flash_attn_ext_vec

Type: TOOL

## From [[drive-research-llamacpp-gpu-memory-reporting-bug|drive-research-llamacpp-gpu-memory-reporting-bug]] (2026-06-08)
- A vectorized FlashAttention kernel.
- Produced incorrect results on AMD RDNA hardware when processing negative infinity limits.
