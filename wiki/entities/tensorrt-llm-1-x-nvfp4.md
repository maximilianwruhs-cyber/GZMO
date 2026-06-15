---
type: entity
title: TensorRT-LLM 1.x NVFP4
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# TensorRT-LLM 1.x NVFP4

Type: TOOL

## From [[aether-grid-micro02|aether-grid-micro02]] (2026-06-09)
- Edge-Inferenz
- currently 1.x-Reihe, v0.10+ from 2024 as base
- NVFP4-Quantisierung, Weight-Streaming, LoRA-Support and Eagle3-Speculative-Decoding
- 2.5x Performance-Gain on DGX Spark (Jan-2026-Update) for models up to 200B
- FP4 + Paged KV-Cache for <200 ms Level-1-Latency
- dynamic loading via NIM/Triton
- enables 2.5x performance gain
- used for local inference up to 200B models
- Local LLM (up to 200B)
- 2.5x Gain Jan-2026
- NVFP4 + LoRA-Dynamic-Loading
- LoRA-Dynamic-Loading
