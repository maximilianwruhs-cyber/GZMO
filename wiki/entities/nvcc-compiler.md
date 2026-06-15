---
type: entity
title: NVCC compiler
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# NVCC compiler

Type: TOOL

## From [[drive-research-ultimate-linux-workstation-tuning-blueprint|drive-research-ultimate-linux-workstation-tuning-blueprint]] (2026-06-08)
- targeted by Nix configuration
- compiles for GB20 Streaming Multiprocessors

## From [[drive-research-llamacpp-optimization-blueprint-micro02|drive-research-llamacpp-optimization-blueprint-micro02]] (2026-06-09)
- The build system generates kernels via this compiler when -DGGML_CUDA=ON is used.
