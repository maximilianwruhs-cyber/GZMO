---
type: entity
title: GGML_SCHED_MAX_SPLIT_INPUTS
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# GGML_SCHED_MAX_SPLIT_INPUTS

Type: CONCEPT

## From [[drive-research-llamacpp-gpu-memory-reporting-bug|drive-research-llamacpp-gpu-memory-reporting-bug]] (2026-06-08)
- Compiled limit for cross-device copy operations.
- Exceeding this limit triggers an assertion failure.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro04|optimizing-nvidia-blackwell-sm120-part3-micro04]] (2026-06-09)
- A compiled limit for cross-device copy operations.
- Exceeding this limit triggers an assertion failure.
- GGML_SCHED_MAX_SPLIT_INPUTS is a constant within ggml.
- ggml_gallocr_alloc_graph is a function that can cause SIGSEGV.
