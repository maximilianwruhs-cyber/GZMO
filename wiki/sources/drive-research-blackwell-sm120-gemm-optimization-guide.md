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
- [NVIDIA Blackwell SM120](/entities/nvidia-blackwell-sm120.md) (SYSTEM)
- [Tensor Memory Accelerator (TMA)](/entities/tensor-memory-accelerator-tma.md) (SYSTEM)
- [Block-scaled GEMM](/entities/block-scaled-gemm.md) (CONCEPT)
- [mx_float8_t](/entities/mx-float8-t.md) (CONCEPT)
- [Mixture of Experts (MoE)](/entities/mixture-of-experts-moe.md) (CONCEPT)
- [SM121](/entities/sm121.md) (SYSTEM)
- [DGX Spark GB10](/entities/dgx-spark-gb10.md) (SYSTEM)
- [mma.sync.aligned.m16n8k64.row.col.kind::mxf8f6f4.block_scale.scale_vec::2X.f32.e2m1.e2m1.f32](/entities/mma-sync-aligned-m16n8k64-row-col-kind-mxf8f6f4-block-scale-scale-vec-2x-f32-e2m1-e2m1-f32.md) (CONCEPT)
- [vLLM](/entities/vllm.md) (TOOL)
- [LPDDR5X](/entities/lpddr5x.md) (SYSTEM)
- [FP6](/entities/fp6.md) (CONCEPT)
- [float_ue8m0_t](/entities/float-ue8m0-t.md) (CONCEPT)
- [CUTLASS](/entities/cutlass.md) (TOOL)
- [nv_float4_t](/entities/nv-float4-t.md) (CONCEPT)
- [Gemma 4](/entities/gemma-4.md) (CONCEPT)
- [TensorRT-LLM](/entities/tensorrt-llm.md) (TOOL)
- [GDDR7](/entities/gddr7.md) (SYSTEM)
- [FlashInfer](/entities/flashinfer.md) (TOOL)
- [CuTe DSL](/entities/cute-dsl.md) (TOOL)
- [Marlin](/entities/marlin.md) (TOOL)
- [MXFP8](/entities/mxfp8.md) (CONCEPT)
- [FP8](/entities/fp8.md) (CONCEPT)
- [HBM3e](/entities/hbm3e.md) (SYSTEM)
- [PTX tcgen05](/entities/ptx-tcgen05.md) (CONCEPT)
- [Hopper](/entities/hopper.md) (SYSTEM)
- [NVFP4](/entities/nvfp4.md) (CONCEPT)
- [SM100](/entities/sm100.md) (SYSTEM)
- [NVCC](/entities/nvcc.md) (TOOL)
- [Turing](/entities/turing.md) (SYSTEM)
- [Warp Group Matrix Multiply (WGMMA)](/entities/warp-group-matrix-multiply-wgmma.md) (CONCEPT)
- [compute_120](/entities/compute-120.md) (CONCEPT)
- [TSMC 4NP](/entities/tsmc-4np.md) (CONCEPT)
- [RTX 5090](/entities/rtx-5090.md) (SYSTEM)
- [Grid Dependency Control (GDC)](/entities/grid-dependency-control-gdc.md) (CONCEPT)
- [FP4](/entities/fp4.md) (CONCEPT)
- [Blackwell Native Microscaling (NV Formats)](/entities/blackwell-native-microscaling-nv-formats.md) (CONCEPT)
- [Cluster Launch Control (CLC)](/entities/cluster-launch-control-clc.md) (SYSTEM)
- [General Matrix Multiply (GEMM)](/entities/general-matrix-multiply-gemm.md) (CONCEPT)
- [Tensor Cores](/entities/tensor-cores.md) (SYSTEM)
- [mx_float_t](/entities/mx-float-t.md) (CONCEPT)
- [mx_float4_t](/entities/mx-float4-t.md) (CONCEPT)
- [RTX PRO 6000](/entities/rtx-pro-6000.md) (SYSTEM)
- [OCP-Compliant Microscaling (MX Formats)](/entities/ocp-compliant-microscaling-mx-formats.md) (CONCEPT)
- [compute_120a](/entities/compute-120a.md) (CONCEPT)
- [PipelineCLCFetchAsync](/entities/pipelineclcfetchasync.md) (TOOL)
- [Tensor Memory (TMEM)](/entities/tensor-memory-tmem.md) (SYSTEM)
- [SGLang](/entities/sglang.md) (TOOL)
- [Ampere](/entities/ampere.md) (SYSTEM)
- [float_ue4m3_t](/entities/float-ue4m3-t.md) (CONCEPT)
- [compute_120f](/entities/compute-120f.md) (CONCEPT)
- [TSMC 4N](/entities/tsmc-4n.md) (CONCEPT)
- [CUDA](/entities/cuda.md) (TOOL)

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
