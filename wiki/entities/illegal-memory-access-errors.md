---
type: entity
title: illegal memory access errors
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# illegal memory access errors

Type: CONCEPT

## From [drive-research-cuda-graph-capture-failure-workarounds-micro03](/entities/drive-research-cuda-graph-capture-failure-workarounds-micro03.md) (2026-06-09)
- Triggered during mathematical reduction operations by large context lengths and high batch sizes.
- Can be avoided by compiling the backend using CCCL with GGML_CUDA_USE_CUB enabled.
