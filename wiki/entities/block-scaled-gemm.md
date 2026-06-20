---
type: entity
title: Block-scaled GEMM
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Block-scaled GEMM

Type: CONCEPT

## From [drive-research-blackwell-sm120-gemm-optimization-guide](/entities/drive-research-blackwell-sm120-gemm-optimization-guide.md) (2026-06-08)
- Resolves dynamic range limitations by partitioning input matrices.
- Applies shared, higher-precision scale factors.
- Two distinct microscaling formats are supported: OCP-Compliant and Blackwell Native.
- Grouped GEMM
- SM120 GEMMs
- SM120 NVF4 GEMM
- SM120 blockwise FP8 GEMM
