---
type: entity
title: CuTe DSL
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# CuTe DSL

Type: TOOL

## From [drive-research-blackwell-sm120-gemm-optimization-guide](/entities/drive-research-blackwell-sm120-gemm-optimization-guide.md) (2026-06-08)
- Python bindings require patching to support sm_120a and sm_120f.
- Partition_S() utility function has an alignment drop bug.

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- Has a bug within the parser in CUTLASS, specifically targeting SM120 FP4 TMA descriptor lowering.
- Lowering mechanics for SM120 FP4 generate a 128-byte tensor-map descriptor payload with byte-level mismatches.
