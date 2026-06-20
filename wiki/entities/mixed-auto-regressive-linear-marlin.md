---
type: entity
title: Mixed Auto-Regressive Linear (Marlin)
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Mixed Auto-Regressive Linear (Marlin)

Type: TOOL

## From [drive-research-imagine-creating-sm120-according-to-our-progress](/entities/drive-research-imagine-creating-sm120-according-to-our-progress.md) (2026-06-08)
- Route pipelines through the emulated Mixed Auto-Regressive Linear (Marlin) kernel.
- Marlin implements dequantization (FP16 x INT4 or FP16 x FP4) entirely in software via inline PTX vector instructions on vector cores.
- It bypasses broken hardware-level TMA, avoids compiler bugs, and yields ideal 4x speedups up to batch sizes of 16 to 32 tokens.
- Speculative decoding must be disabled when running under the emulated Marlin path.
- Enforce global Marlin routing.
- Tier 1: Stable Fallback Path (Marlin Dequantization Baseline).
- For immediate production stability without runtime crashes, route pipelines through the emulated Mixed Auto-Regressive Linear (Marlin) kernel.

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- An emulated dequantization pipeline serving as a fallback baseline for SM120 deployments.
- An extremely optimized mixed-precision matrix multiplication kernel.
- Capable of executing FP16 x INT4 or FP16 x FP4 operations.
- Implements dequantization entirely in software.
- Requires a minimum compute capability of >= 8.0.
- Relies on inline PTX assembly for custom vector decompression instructions.
- Optimizes only GEMM operations.
- When used with speculative decoding on SM120, results in a performance regression of up to -22%.
- A critical, high-performance fallback baseline for SM120 deployments.
- Designed as an optimized mixed-precision matrix multiplication kernel.
- Achieves close to ideal 4x speedups up to batch sizes of 16 to 32 tokens.
- Relies on three distinct software and architectural techniques: Asynchronous Global Memory Loads, Double-Buffered Pipelining, and L2 Cache Bypassing.
- Uses a dedicated layer packer to prepare weight matrices.
- Relies on Asynchronous Global Memory Loads, Double-Buffered Pipelining, and L2 Cache Bypassing.
