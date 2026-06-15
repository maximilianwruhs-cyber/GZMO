---
type: source
title: drive-research-blackwell-sm120-gemm-optimization-guide
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-blackwell-sm120-gemm-optimization-guide

Ingested source summary (2026-06-08).

## Entities
- [[nvidia-blackwell-sm120|NVIDIA Blackwell SM120]] (SYSTEM)
- [[tensor-memory-accelerator-tma|Tensor Memory Accelerator (TMA)]] (SYSTEM)
- [[block-scaled-gemm|Block-scaled GEMM]] (CONCEPT)
- [[mx-float8-t|mx_float8_t]] (CONCEPT)
- [[mixture-of-experts-moe|Mixture of Experts (MoE)]] (CONCEPT)
- [[sm121|SM121]] (SYSTEM)
- [[dgx-spark-gb10|DGX Spark GB10]] (SYSTEM)
- [[mma-sync-aligned-m16n8k64-row-col-kind-mxf8f6f4-block-scale-scale-vec-2x-f32-e2m1-e2m1-f32|mma.sync.aligned.m16n8k64.row.col.kind::mxf8f6f4.block_scale.scale_vec::2X.f32.e2m1.e2m1.f32]] (CONCEPT)
- [[vllm|vLLM]] (TOOL)
- [[lpddr5x|LPDDR5X]] (SYSTEM)
- [[fp6|FP6]] (CONCEPT)
- [[float-ue8m0-t|float_ue8m0_t]] (CONCEPT)
- [[cutlass|CUTLASS]] (TOOL)
- [[nv-float4-t|nv_float4_t]] (CONCEPT)
- [[gemma-4|Gemma 4]] (CONCEPT)
- [[tensorrt-llm|TensorRT-LLM]] (TOOL)
- [[gddr7|GDDR7]] (SYSTEM)
- [[flashinfer|FlashInfer]] (TOOL)
- [[cute-dsl|CuTe DSL]] (TOOL)
- [[marlin|Marlin]] (TOOL)
- [[mxfp8|MXFP8]] (CONCEPT)
- [[fp8|FP8]] (CONCEPT)
- [[hbm3e|HBM3e]] (SYSTEM)
- [[ptx-tcgen05|PTX tcgen05]] (CONCEPT)
- [[hopper|Hopper]] (SYSTEM)
- [[nvfp4|NVFP4]] (CONCEPT)
- [[sm100|SM100]] (SYSTEM)
- [[nvcc|NVCC]] (TOOL)
- [[turing|Turing]] (SYSTEM)
- [[warp-group-matrix-multiply-wgmma|Warp Group Matrix Multiply (WGMMA)]] (CONCEPT)
- [[compute-120|compute_120]] (CONCEPT)
- [[tsmc-4np|TSMC 4NP]] (CONCEPT)
- [[rtx-5090|RTX 5090]] (SYSTEM)
- [[grid-dependency-control-gdc|Grid Dependency Control (GDC)]] (CONCEPT)
- [[fp4|FP4]] (CONCEPT)
- [[blackwell-native-microscaling-nv-formats|Blackwell Native Microscaling (NV Formats)]] (CONCEPT)
- [[cluster-launch-control-clc|Cluster Launch Control (CLC)]] (SYSTEM)
- [[general-matrix-multiply-gemm|General Matrix Multiply (GEMM)]] (CONCEPT)
- [[tensor-cores|Tensor Cores]] (SYSTEM)
- [[mx-float-t|mx_float_t]] (CONCEPT)
- [[mx-float4-t|mx_float4_t]] (CONCEPT)
- [[rtx-pro-6000|RTX PRO 6000]] (SYSTEM)
- [[ocp-compliant-microscaling-mx-formats|OCP-Compliant Microscaling (MX Formats)]] (CONCEPT)
- [[compute-120a|compute_120a]] (CONCEPT)
- [[pipelineclcfetchasync|PipelineCLCFetchAsync]] (TOOL)
- [[tensor-memory-tmem|Tensor Memory (TMEM)]] (SYSTEM)
- [[sglang|SGLang]] (TOOL)
- [[ampere|Ampere]] (SYSTEM)
- [[float-ue4m3-t|float_ue4m3_t]] (CONCEPT)
- [[compute-120f|compute_120f]] (CONCEPT)
- [[tsmc-4n|TSMC 4N]] (CONCEPT)
- [[cuda|CUDA]] (TOOL)

## Relations
- NVIDIA Blackwell SM120 → RELATED_TO → General Matrix Multiply (GEMM)
- NVIDIA Blackwell SM120 → RELATED_TO → SM100
- NVIDIA Blackwell SM120 → RELATED_TO → SM121
- SM100 → USES → Tensor Cores
- SM100 → USES → PTX tcgen05
- SM100 → PART_OF → Tensor Memory (TMEM)
- NVIDIA Blackwell SM120 → USES → mma.sync.aligned.m16n8k64.row.col.kind::mxf8f6f4.block_scale.scale_vec::2X.f32.e2m1.e2m1.f32
- SM121 → USES → mma.sync.aligned.m16n8k64.row.col.kind::mxf8f6f4.block_scale.scale_vec::2X.f32.e2m1.e2m1.f32
- NVIDIA Blackwell SM120 → RELATED_TO → Ampere
- NVIDIA Blackwell SM120 → RELATED_TO → Turing
- SM121 → RELATED_TO → Ampere
- SM121 → RELATED_TO → Turing
- RTX 5090 → PART_OF → NVIDIA Blackwell SM120
- RTX 5090 → PART_OF → SM121
- RTX PRO 6000 → PART_OF → NVIDIA Blackwell SM120
- RTX PRO 6000 → PART_OF → SM121
- DGX Spark GB10 → PART_OF → NVIDIA Blackwell SM120
- DGX Spark GB10 → PART_OF → SM121
- GDDR7 → PART_OF → RTX PRO 6000
- Block-scaled GEMM → RELATED_TO → FP4
- Block-scaled GEMM → RELATED_TO → FP6
- OCP-Compliant Microscaling (MX Formats) → RELATED_TO → mx_float8_t
- OCP-Compliant Microscaling (MX Formats) → RELATED_TO → mx_float_t
- OCP-Compliant Microscaling (MX Formats) → RELATED_TO → mx_float4_t
- OCP-Compliant Microscaling (MX Formats) → USES → float_ue8m0_t
- Blackwell Native Microscaling (NV Formats) → RELATED_TO → nv_float4_t
- nv_float4_t → USES → float_ue4m3_t
- CUTLASS → RELATED_TO → NVIDIA Blackwell SM120
- CUTLASS → RELATED_TO → General Matrix Multiply (GEMM)
- CUTLASS → USES → PTX tcgen05
- CUTLASS → USES → Grid Dependency Control (GDC)
- CUTLASS → USES → PipelineCLCFetchAsync
- CUDA → RELATED_TO → NVCC
- NVCC → USES → compute_120
- NVCC → USES → compute_120a
- NVCC → USES → compute_120f
- compute_120 → RELATED_TO → CUTLASS
- compute_120a → RELATED_TO → CUTLASS
- compute_120f → RELATED_TO → CUTLASS
- Tensor Memory Accelerator (TMA) → USES → PTX tcgen05
- Marlin → RELATED_TO → General Matrix Multiply (GEMM)
- FlashInfer → USES → CUTLASS
- vLLM → USES → CUTLASS
- Grid Dependency Control (GDC) → RELATED_TO → NVIDIA Blackwell SM120
- TensorRT-LLM → RELATED_TO → NVIDIA Blackwell SM120
- TensorRT-LLM → RELATED_TO → SM121
- CuTe DSL → RELATED_TO → NVIDIA Blackwell SM120
- CuTe DSL → RELATED_TO → SM121
- Gemma 4 → RELATED_TO → FP4
- Mixture of Experts (MoE) → RELATED_TO → General Matrix Multiply (GEMM)
- Cluster Launch Control (CLC) → RELATED_TO → NVIDIA Blackwell SM120
- Cluster Launch Control (CLC) → USES → PTX tcgen05
- PipelineCLCFetchAsync → RELATED_TO → Cluster Launch Control (CLC)
- RTX PRO 6000 → PART_OF → GDDR7
- SM100 → RELATED_TO → TSMC 4NP
- NVIDIA Blackwell SM120 → RELATED_TO → TSMC 4N
- SM121 → RELATED_TO → TSMC 4N
- NVIDIA Blackwell SM120 → RELATED_TO → GDDR7
- NVIDIA Blackwell SM120 → RELATED_TO → LPDDR5X
- SM121 → RELATED_TO → GDDR7
- SM121 → RELATED_TO → LPDDR5X
- HBM3e → RELATED_TO → SM100
- NVIDIA Blackwell SM120 → RELATED_TO → NVFP4
- NVIDIA Blackwell SM120 → RELATED_TO → Block-scaled GEMM
- NVIDIA Blackwell SM120 → RELATED_TO → CUDA
- NVIDIA Blackwell SM120 → RELATED_TO → DGX Spark GB10
- NVIDIA Blackwell SM120 → RELATED_TO → Hopper
- NVFP4 → RELATED_TO → Mixture of Experts (MoE)
- NVFP4 → RELATED_TO → FP4
- CUTLASS → RELATED_TO → Block-scaled GEMM
- CUTLASS → PART_OF → NVIDIA Blackwell SM120
- CUTLASS → USES → CUDA
- CUTLASS → RELATED_TO → Tensor Memory Accelerator (TMA)
- Block-scaled GEMM → RELATED_TO → NVIDIA Blackwell SM120
- Block-scaled GEMM → RELATED_TO → FP8
- CUDA → RELATED_TO → NVIDIA Blackwell SM120
- DGX Spark GB10 → RELATED_TO → NVIDIA Blackwell SM120
- MXFP8 → RELATED_TO → NVFP4
- FP4 → RELATED_TO → NVFP4
- FP8 → RELATED_TO → Block-scaled GEMM
- Mixture of Experts (MoE) → RELATED_TO → NVFP4
- NVIDIA Blackwell SM120 → RELATED_TO → CUTLASS
- vLLM → RELATED_TO → NVFP4
- vLLM → RELATED_TO → NVIDIA Blackwell SM120
- Hopper → RELATED_TO → NVIDIA Blackwell SM120
- FlashInfer → RELATED_TO → NVIDIA Blackwell SM120
- Tensor Cores → RELATED_TO → NVIDIA Blackwell SM120
