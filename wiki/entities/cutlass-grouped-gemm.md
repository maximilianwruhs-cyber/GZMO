---
type: entity
title: CUTLASS grouped GEMM
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# CUTLASS grouped GEMM

Type: CONCEPT

## From [[optimizing-nvidia-blackwell-sm120-part1-micro04|optimizing-nvidia-blackwell-sm120-part1-micro04]] (2026-06-09)
- Tactics are hardcoded to assume SM100-class hardware parameters.
- Require 228 KiB or more of shared memory for multi-stage pipelines.
- Fail hardware compatibility check on SM120 due to shared memory limits.
