---
type: entity
title: FlashInfer JIT compiler
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# FlashInfer JIT compiler

Type: TOOL

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- Contains critical bugs when deploying MoE models using native NVFP4 weight formats on SM120.
- Autotuner attempts to initialize TMA Warp-Specialized tactics, which fail hardware compatibility checks on SM120.
- Parses raw environment strings directly during initialization, forcing slow compute_120a fallback path.
