---
type: source
title: optimizing-nvidia-blackwell-sm120-part1-micro04
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# optimizing-nvidia-blackwell-sm120-part1-micro04

Ingested source summary (2026-06-09).

## Entities
- [Mixed Auto-Regressive Linear (Marlin)](/entities/mixed-auto-regressive-linear-marlin.md) (TOOL)
- [FlashInfer JIT compiler](/entities/flashinfer-jit-compiler.md) (TOOL)
- [B200](/entities/b200.md) (SYSTEM)
- [NVIDIA Blackwell architecture](/entities/nvidia-blackwell-architecture.md) (SYSTEM)
- [cuda::memcpy_async](/entities/cuda-memcpy-async.md) (TOOL)
- [CUTLASS_ENABLE_GDC_FOR_SM90=1](/entities/cutlass-enable-gdc-for-sm90-1.md) (TOOL)
- [compute_120](/entities/compute-120.md) (CONCEPT)
- [Ampere architectures](/entities/ampere-architectures.md) (SYSTEM)
- [CuTe DSL](/entities/cute-dsl.md) (TOOL)
- [SM120 architecture](/entities/sm120-architecture.md) (SYSTEM)
- [RTX 5090](/entities/rtx-5090.md) (SYSTEM)
- [CUTLASS grouped GEMM](/entities/cutlass-grouped-gemm.md) (CONCEPT)
- [INT4](/entities/int4.md) (CONCEPT)
- [Tensor Memory (TMEM)](/entities/tensor-memory-tmem.md) (SYSTEM)
- [Speculative Decoding](/entities/speculative-decoding.md) (CONCEPT)
- [compute_120a](/entities/compute-120a.md) (CONCEPT)
- [GDDR7 memory bus](/entities/gddr7-memory-bus.md) (SYSTEM)
- [CUTLASS_GDC_ENABLED](/entities/cutlass-gdc-enabled.md) (TOOL)
- [CUTLASS_ENABLE_GDC_FOR_SM100=1](/entities/cutlass-enable-gdc-for-sm100-1.md) (TOOL)
- [CUTLASS library](/entities/cutlass-library.md) (TOOL)
- [MTP drafter](/entities/mtp-drafter.md) (SYSTEM)
- [DGX Spark GB10](/entities/dgx-spark-gb10.md) (SYSTEM)
- [Multi-Token Prediction (MTP)](/entities/multi-token-prediction-mtp.md) (CONCEPT)
- [TensorRT-LLM](/entities/tensorrt-llm.md) (TOOL)
- [Ada architectures](/entities/ada-architectures.md) (SYSTEM)
- [Tensor Memory Accelerator (TMA)](/entities/tensor-memory-accelerator-tma.md) (SYSTEM)
- [FP4](/entities/fp4.md) (CONCEPT)
- [B100](/entities/b100.md) (SYSTEM)
- [Grid Dependency Control (GDC)](/entities/grid-dependency-control-gdc.md) (CONCEPT)
- [Layer Packer](/entities/layer-packer.md) (TOOL)
- [NVFP4 format](/entities/nvfp4-format.md) (CONCEPT)
- [cuda::pipeline](/entities/cuda-pipeline.md) (TOOL)
- [Warp-Level Matrix Multiply-Accumulate (WMMA) 16x16x16](/entities/warp-level-matrix-multiply-accumulate-wmma-16x16x16.md) (CONCEPT)
- [vLLM](/entities/vllm.md) (SYSTEM)
- [FP16](/entities/fp16.md) (CONCEPT)
- [tcgen05 instruction set architecture](/entities/tcgen05-instruction-set-architecture.md) (SYSTEM)
- [RTX PRO 6000 Blackwell Workstation Edition](/entities/rtx-pro-6000-blackwell-workstation-edition.md) (SYSTEM)
- [SM100 family](/entities/sm100-family.md) (SYSTEM)
- [compute_120f](/entities/compute-120f.md) (CONCEPT)
- [HBM3e](/entities/hbm3e.md) (SYSTEM)
- [FLASHINFER_CUDA_ARCH_LIST=12.0a](/entities/flashinfer-cuda-arch-list-12-0a.md) (TOOL)
- [SM121 variant](/entities/sm121-variant.md) (SYSTEM)

## Relations
- NVIDIA Blackwell architecture → PART_OF → SM100 family
- NVIDIA Blackwell architecture → PART_OF → SM120 architecture
- NVIDIA Blackwell architecture → PART_OF → SM121 variant
- SM100 family → PART_OF → B100
- SM100 family → PART_OF → B200
- SM100 family → USES → tcgen05 instruction set architecture
- SM100 family → USES → HBM3e
- SM120 architecture → USES → GDDR7 memory bus
- SM120 architecture → RELATED_TO → NVFP4 format
- SM121 variant → USES → GDDR7 memory bus
- SM121 variant → RELATED_TO → NVFP4 format
- SM100 family → USES → Tensor Memory (TMEM)
- SM120 architecture → USES → Tensor Memory (TMEM)
- SM121 variant → USES → Tensor Memory (TMEM)
- CUTLASS library → USES → SM120 architecture
- CUTLASS library → RELATED_TO → NVFP4 format
- FlashInfer JIT compiler → USES → SM120 architecture
- FlashInfer JIT compiler → RELATED_TO → NVFP4 format
- Tensor Memory Accelerator (TMA) → RELATED_TO → SM100 family
- Tensor Memory Accelerator (TMA) → RELATED_TO → SM120 architecture
- CUTLASS grouped GEMM → RELATED_TO → SM100 family
- CUTLASS grouped GEMM → RELATED_TO → SM120 architecture
- Grid Dependency Control (GDC) → PART_OF → CUTLASS library
- Grid Dependency Control (GDC) → RELATED_TO → SM120 architecture
- CuTe DSL → PART_OF → CUTLASS library
- CuTe DSL → RELATED_TO → SM120 architecture
- vLLM → USES → SM120 architecture
- vLLM → USES → SM121 variant
- TensorRT-LLM → USES → vLLM
- TensorRT-LLM → RELATED_TO → SM120 architecture
- TensorRT-LLM → RELATED_TO → SM121 variant
- Mixed Auto-Regressive Linear (Marlin) → RELATED_TO → SM120 architecture
- Mixed Auto-Regressive Linear (Marlin) → USES → FP16
- Mixed Auto-Regressive Linear (Marlin) → USES → INT4
- Mixed Auto-Regressive Linear (Marlin) → USES → FP4
- Mixed Auto-Regressive Linear (Marlin) → RELATED_TO → Ampere architectures
- Mixed Auto-Regressive Linear (Marlin) → RELATED_TO → Ada architectures
- Mixed Auto-Regressive Linear (Marlin) → RELATED_TO → Tensor Memory Accelerator (TMA)
- Mixed Auto-Regressive Linear (Marlin) → USES → cuda::memcpy_async
- Mixed Auto-Regressive Linear (Marlin) → USES → cuda::pipeline
- Multi-Token Prediction (MTP) → RELATED_TO → NVFP4 format
- Speculative Decoding → RELATED_TO → Mixed Auto-Regressive Linear (Marlin)
- Speculative Decoding → RELATED_TO → SM120 architecture
- Speculative Decoding → RELATED_TO → MTP drafter
- FP16 → RELATED_TO → Mixed Auto-Regressive Linear (Marlin)
- FP4 → RELATED_TO → Mixed Auto-Regressive Linear (Marlin)
- FP4 → RELATED_TO → SM120 architecture
- cuda::memcpy_async → PART_OF → Mixed Auto-Regressive Linear (Marlin)
- cuda::pipeline → PART_OF → Mixed Auto-Regressive Linear (Marlin)
- CUTLASS_ENABLE_GDC_FOR_SM100=1 → RELATED_TO → SM120 architecture
- CUTLASS_ENABLE_GDC_FOR_SM90=1 → RELATED_TO → SM120 architecture
- CUTLASS_GDC_ENABLED → RELATED_TO → SM120 architecture
- compute_120 → RELATED_TO → SM120 architecture
- compute_120a → RELATED_TO → SM120 architecture
- compute_120f → RELATED_TO → SM120 architecture
- FLASHINFER_CUDA_ARCH_LIST=12.0a → USES → FlashInfer JIT compiler
- FLASHINFER_CUDA_ARCH_LIST=12.0a → RELATED_TO → SM120 architecture
- Layer Packer → PART_OF → Mixed Auto-Regressive Linear (Marlin)
- MTP drafter → RELATED_TO → Mixed Auto-Regressive Linear (Marlin)
