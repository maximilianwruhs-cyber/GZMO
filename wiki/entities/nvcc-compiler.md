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

## From [drive-research-ultimate-linux-workstation-tuning-blueprint](/entities/drive-research-ultimate-linux-workstation-tuning-blueprint.md) (2026-06-08)
- targeted by Nix configuration
- compiles for GB20 Streaming Multiprocessors

## From [drive-research-llamacpp-optimization-blueprint-micro02](/entities/drive-research-llamacpp-optimization-blueprint-micro02.md) (2026-06-09)
- The build system generates kernels via this compiler when -DGGML_CUDA=ON is used.
