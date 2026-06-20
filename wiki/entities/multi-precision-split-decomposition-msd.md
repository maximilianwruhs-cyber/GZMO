---
type: entity
title: Multi-Precision Split Decomposition (MSD)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Multi-Precision Split Decomposition (MSD)

Type: CONCEPT

## From [drive-research-what-else-can-directly-be-aligned-with-our-common](/entities/drive-research-what-else-can-directly-be-aligned-with-our-common.md) (2026-06-08)
- Avoids register pressure and compute bottlenecks on SM120 streaming multiprocessors by decomposing high-precision (BF16) activations into multiple low-precision components.
- Split components are multiplied directly with quantized weights using native Tensor Core hardware instructions.
