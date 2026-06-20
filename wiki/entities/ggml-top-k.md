---
type: entity
title: ggml_top_k
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# ggml_top_k

Type: TOOL

## From [drive-research-cuda-graph-capture-failure-workarounds-micro02](/entities/drive-research-cuda-graph-capture-failure-workarounds-micro02.md) (2026-06-09)
- Custom kernel that can exceed index boundaries on exceptionally large tensor dimensions.
- Thread indexing logic fails to check boundary limits.

## From [optimizing-nvidia-blackwell-sm120-part3-micro05](/entities/optimizing-nvidia-blackwell-sm120-part3-micro05.md) (2026-06-09)
- Custom mathematical operator.
- Can exceed index boundaries on exceptionally large tensor dimensions.
- Thread indexing logic fails to check boundary limits.
