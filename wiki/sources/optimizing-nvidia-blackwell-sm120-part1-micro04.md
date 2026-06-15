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
- [[mixed-auto-regressive-linear-marlin|Mixed Auto-Regressive Linear (Marlin)]] (TOOL)
- [[flashinfer-jit-compiler|FlashInfer JIT compiler]] (TOOL)
- [[b200|B200]] (SYSTEM)
- [[nvidia-blackwell-architecture|NVIDIA Blackwell architecture]] (SYSTEM)
- [[cuda-memcpy-async|cuda::memcpy_async]] (TOOL)
- [[cutlass-enable-gdc-for-sm90-1|CUTLASS_ENABLE_GDC_FOR_SM90=1]] (TOOL)
- [[compute-120|compute_120]] (CONCEPT)
- [[ampere-architectures|Ampere architectures]] (SYSTEM)
- [[cute-dsl|CuTe DSL]] (TOOL)
- [[sm120-architecture|SM120 architecture]] (SYSTEM)
- [[rtx-5090|RTX 5090]] (SYSTEM)
- [[cutlass-grouped-gemm|CUTLASS grouped GEMM]] (CONCEPT)
- [[int4|INT4]] (CONCEPT)
- [[tensor-memory-tmem|Tensor Memory (TMEM)]] (SYSTEM)
- [[speculative-decoding|Speculative Decoding]] (CONCEPT)
- [[compute-120a|compute_120a]] (CONCEPT)
- [[gddr7-memory-bus|GDDR7 memory bus]] (SYSTEM)
- [[cutlass-gdc-enabled|CUTLASS_GDC_ENABLED]] (TOOL)
- [[cutlass-enable-gdc-for-sm100-1|CUTLASS_ENABLE_GDC_FOR_SM100=1]] (TOOL)
- [[cutlass-library|CUTLASS library]] (TOOL)
- [[mtp-drafter|MTP drafter]] (SYSTEM)
- [[dgx-spark-gb10|DGX Spark GB10]] (SYSTEM)
- [[multi-token-prediction-mtp|Multi-Token Prediction (MTP)]] (CONCEPT)
- [[tensorrt-llm|TensorRT-LLM]] (TOOL)
- [[ada-architectures|Ada architectures]] (SYSTEM)
- [[tensor-memory-accelerator-tma|Tensor Memory Accelerator (TMA)]] (SYSTEM)
- [[fp4|FP4]] (CONCEPT)
- [[b100|B100]] (SYSTEM)
- [[grid-dependency-control-gdc|Grid Dependency Control (GDC)]] (CONCEPT)
- [[layer-packer|Layer Packer]] (TOOL)
- [[nvfp4-format|NVFP4 format]] (CONCEPT)
- [[cuda-pipeline|cuda::pipeline]] (TOOL)
- [[warp-level-matrix-multiply-accumulate-wmma-16x16x16|Warp-Level Matrix Multiply-Accumulate (WMMA) 16x16x16]] (CONCEPT)
- [[vllm|vLLM]] (SYSTEM)
- [[fp16|FP16]] (CONCEPT)
- [[tcgen05-instruction-set-architecture|tcgen05 instruction set architecture]] (SYSTEM)
- [[rtx-pro-6000-blackwell-workstation-edition|RTX PRO 6000 Blackwell Workstation Edition]] (SYSTEM)
- [[sm100-family|SM100 family]] (SYSTEM)
- [[compute-120f|compute_120f]] (CONCEPT)
- [[hbm3e|HBM3e]] (SYSTEM)
- [[flashinfer-cuda-arch-list-12-0a|FLASHINFER_CUDA_ARCH_LIST=12.0a]] (TOOL)
- [[sm121-variant|SM121 variant]] (SYSTEM)

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
