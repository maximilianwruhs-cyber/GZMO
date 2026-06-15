---
type: entity
title: mma.sync.aligned.m16n8k64.row.col.kind::mxf8f6f4.block_scale.scale_vec::2X.f32.e2m1.e2m1.f32
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# mma.sync.aligned.m16n8k64.row.col.kind::mxf8f6f4.block_scale.scale_vec::2X.f32.e2m1.e2m1.f32

Type: CONCEPT

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- Extended flavor used by SM120/121.
- Operates similarly to execution patterns in Ampere and Turing architectures.
- Primary PTX instruction for block-scaled FP4 matrix multiplication on SM120.
- Processes a matrix tile with dimensions M=16, N=8, K=64.
- Uses FP4 e2m1 representation for input matrices A and B.
