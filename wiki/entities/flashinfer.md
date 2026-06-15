---
type: entity
title: FlashInfer
created: 2026-06-08
updated: 2026-06-10
sources: 9
tags: []
status: draft
gzmo_synthetic: true
---









# FlashInfer

Type: TOOL

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- Uses CUTLASS.
- Native FP4 implementation performance is significantly improved with compute_120f target.
- FlashInfer SM120 Patches

## From [[drive-research-flashinfer-moe-fp4-jit-error|drive-research-flashinfer-moe-fp4-jit-error]] (2026-06-08)
- A backend engine for MoE models
- Encounters JIT compilation errors on workstation Blackwell
- Requires patches for stable native FP4 execution

## From [[drive-research-llm-inference-engine-audit-2026-micro02|drive-research-llm-inference-engine-audit-2026-micro02]] (2026-06-09)
- Efficient and Customizable Attention Engine for LLM Inference Serving.

## From [[drive-research-marlin-baseline-for-early-deployments-micro02|drive-research-marlin-baseline-for-early-deployments-micro02]] (2026-06-09)
- Has a native FP4 MoE path that can be broken.
- Kernel selection can be disabled via environment variables.

## From [[drive-research-llm-inference-engine-audit-2026-micro01|drive-research-llm-inference-engine-audit-2026-micro01]] (2026-06-10)
- Definitive attention engine library for 2026
- Provides unified APIs for attention and matrix multiplication

## From [[drive-research-marlin-baseline-for-early-deployments-micro01|drive-research-marlin-baseline-for-early-deployments-micro01]] (2026-06-10)
- JIT compiler used for kernels.
- Contains JIT compiler templates that fail on SM120 hardware.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro05|optimizing-nvidia-blackwell-sm120-part1-micro05]] (2026-06-10)
- Contains a broken native FP4 MoE kernel pathway on SM120
- Requires patching for SM120 support

## From [[optimizing-nvidia-blackwell-sm120-part1-micro07|optimizing-nvidia-blackwell-sm120-part1-micro07]] (2026-06-10)
- Requires clearing JIT compilation caches (rm -rf ~/.cache/flashinfer/*) after implementing modifications.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro02|optimizing-nvidia-blackwell-sm120-part2-micro02]] (2026-06-10)
- Execution backend used for benchmarking CUTLASS implementations.
