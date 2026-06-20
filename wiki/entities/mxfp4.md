---
type: entity
title: MXFP4
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# MXFP4

Type: TOOL

## From [drive-research-speicherbandbreiten-engpass-memory-wall](/entities/drive-research-speicherbandbreiten-engpass-memory-wall.md) (2026-06-08)
- A Weight-Only-Quantized (WOQ) model.
- Used as the primary draft in ML-SpecQD.
- Contributes to achieving speedups over baseline BF16 inference.

## From [drive-research-llamacpp-optimization-blueprint-micro02](/entities/drive-research-llamacpp-optimization-blueprint-micro02.md) (2026-06-09)
- Native support for this was introduced for Blackwell-class GPUs.
- Passing -DGGML_NATIVE=OFF while retaining GGML_CUDA=ON bypasses the MXFP4 native support.
