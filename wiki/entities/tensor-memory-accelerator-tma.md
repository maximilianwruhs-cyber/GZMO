---
type: entity
title: Tensor Memory Accelerator (TMA)
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# Tensor Memory Accelerator (TMA)

Type: SYSTEM

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- Hardware on Blackwell.
- Relies on PTX prefetch.tensormap instruction.
- Descriptor addresses must be strictly aligned to 64-byte boundaries.
- CUTLASS's Params structures were declared without explicit alignment attributes, causing crashes.
- TMA descriptor alignment fix
- TMA descriptor misalignment

## From [[drive-research-imagine-creating-sm120-according-to-our-progress|drive-research-imagine-creating-sm120-according-to-our-progress]] (2026-06-08)
- Tensor Memory Accelerator (TMA) Warp-Specialized (WS) tactics fail hardware compatibility checks at runtime.
- The fastest TMA Warp-Specialized grouped GEMM tactics to execute safely.
- TMA and Warp-Specialized instructions enforce strict alignment boundaries on physical memory addresses.
- TMA Descriptor Alignment: The PTX instruction prefetch.tensormap requires that parameter structures representing TMA descriptors be strictly aligned to 64-byte boundaries.
- TMA Load Descriptors patch.
- TMA Warp-Specialized grouped GEMM tactics.

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- A hardware feature for zero-overhead global-to-shared memory transfers.
- Requires strict, low-level alignment boundaries on physical memory addresses.
- The PTX instruction prefetch.tensormap requires TMA descriptors to be strictly aligned to 64-byte boundaries.
- Parameter structures must be patched with explicit alignment attributes (alignas(64)) to satisfy physical requirements.
- A hardware feature supported by SM120 and SM121.
- Used for zero-overhead global-to-shared memory transfers.
- Requires strict alignment boundaries.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro04|optimizing-nvidia-blackwell-sm120-part1-micro04]] (2026-06-09)
- Warp-Specialized (WS) tactics require >= 228KB shared memory.
- TMA WS tactics fail hardware compatibility checks on SM120.
- Native FP4 pathways rely on broken hardware-level TMA.
- Marlin bypasses broken TMA structures.
