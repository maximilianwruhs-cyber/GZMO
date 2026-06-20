---
type: source
title: drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01

Ingested source summary (2026-06-09).

## Entities
- [drive_clean/Takeout/Drive/Optimizing CUDA Performance with FP4_FP6.docx](/entities/drive-clean-takeout-drive-optimizing-cuda-performance-with-fp4-fp6-docx.md) (BOOK)
- [CUTLASS autotuner](/entities/cutlass-autotuner.md) (TOOL)
- [Engineering High-Throughput Sub-Byte Quantized Inference on NVIDIA Blackwell Architectures: A Deep Dive into CUDA 13.1 C](/entities/engineering-high-throughput-sub-byte-quantized-inference-on-nvidia-blackwell-architectures-a-deep-dive-into-cuda-13-1-c.md) (BOOK)
- [sm120_mma_tma_blockwise_scaling.hpp](/entities/sm120-mma-tma-blockwise-scaling-hpp.md) (BOOK)
- [cuBLASLtMatmul()](/entities/cublasltmatmul.md) (CONCEPT)
- [B200](/entities/b200.md) (SYSTEM)
- [NVIDIA Blackwell GPU architecture](/entities/nvidia-blackwell-gpu-architecture.md) (SYSTEM)
- [ldmatrix.sync.aligned](/entities/ldmatrix-sync-aligned.md) (CONCEPT)
- [cute::array_aligned](/entities/cute-array-aligned.md) (SYSTEM)
- [INT8](/entities/int8.md) (CONCEPT)
- [CUDA 12.8](/entities/cuda-12-8.md) (SYSTEM)
- [CuTe Domain-Specific Language (DSL)](/entities/cute-domain-specific-language-dsl.md) (TOOL)
- [smem_SFB](/entities/smem-sfb.md) (SYSTEM)
- [GB200](/entities/gb200.md) (SYSTEM)
- [wave_04_drive_research](/entities/wave-04-drive-research.md) (CONCEPT)
- [smem_B](/entities/smem-b.md) (SYSTEM)
- [LPDDR5x](/entities/lpddr5x.md) (SYSTEM)
- [SGLang](/entities/sglang.md) (SYSTEM)
- [Programmatic Dependency Launch (PDL)](/entities/programmatic-dependency-launch-pdl.md) (CONCEPT)
- [smem_A](/entities/smem-a.md) (SYSTEM)
- [Google Takeout](/entities/google-takeout.md) (TOOL)
- [SM121](/entities/sm121.md) (SYSTEM)
- [cuBLAS](/entities/cublas.md) (SYSTEM)
- [smem_SFA](/entities/smem-sfa.md) (SYSTEM)
- [TF32](/entities/tf32.md) (CONCEPT)
- [compute_120f](/entities/compute-120f.md) (CONCEPT)
- [NVRTC](/entities/nvrtc.md) (TOOL)
- [CUDA 13.x](/entities/cuda-13-x.md) (SYSTEM)
- [TensorRT](/entities/tensorrt.md) (SYSTEM)
- [Vision Transformer (ViT)](/entities/vision-transformer-vit.md) (CONCEPT)
- [PTX](/entities/ptx.md) (SYSTEM)
- [TensorStorage](/entities/tensorstorage.md) (SYSTEM)
- [SM120 Blackwell platforms](/entities/sm120-blackwell-platforms.md) (SYSTEM)
- [cute::ArrayEngine](/entities/cute-arrayengine.md) (SYSTEM)
- [compute_120a](/entities/compute-120a.md) (CONCEPT)
- [MSVC](/entities/msvc.md) (TOOL)
- [CUTLASS templates](/entities/cutlass-templates.md) (SYSTEM)
- [CUDA 13.1 Update 2](/entities/cuda-13-1-update-2.md) (SYSTEM)
- [Hopper architectures](/entities/hopper-architectures.md) (SYSTEM)
- [sm120_blockscaled_mma_tma.hpp](/entities/sm120-blockscaled-mma-tma-hpp.md) (BOOK)
- [FLUX](/entities/flux.md) (PROJECT)
- [CUDA Toolkit 13.1](/entities/cuda-toolkit-13-1.md) (SYSTEM)
- [RTX 5090](/entities/rtx-5090.md) (SYSTEM)
- [x86_64](/entities/x86-64.md) (SYSTEM)
- [Tensor Memory Accelerator (TMA)](/entities/tensor-memory-accelerator-tma.md) (SYSTEM)
- [GDDR7](/entities/gddr7.md) (SYSTEM)
- [llama.cpp](/entities/llama-cpp.md) (SYSTEM)
- [compute_121a](/entities/compute-121a.md) (CONCEPT)
- [thread block clusters](/entities/thread-block-clusters.md) (CONCEPT)
- [Tensor Core formats](/entities/tensor-core-formats.md) (CONCEPT)
- [B100](/entities/b100.md) (SYSTEM)
- [CUDA 13.0](/entities/cuda-13-0.md) (SYSTEM)
- [DGX Spark](/entities/dgx-spark.md) (SYSTEM)
- [FP16](/entities/fp16.md) (CONCEPT)
- [Warp-Specialized MMA instructions](/entities/warp-specialized-mma-instructions.md) (CONCEPT)
- [compute_120](/entities/compute-120.md) (CONCEPT)
- [SM100](/entities/sm100.md) (SYSTEM)
- [aarch64](/entities/aarch64.md) (SYSTEM)
- [CUDA 12.9](/entities/cuda-12-9.md) (SYSTEM)
- [partition_S()](/entities/partition-s.md) (CONCEPT)
- [FP8](/entities/fp8.md) (CONCEPT)
- [RTX PRO 6000](/entities/rtx-pro-6000.md) (SYSTEM)
- [BF16](/entities/bf16.md) (CONCEPT)
- [vLLM](/entities/vllm.md) (SYSTEM)
- [Ada](/entities/ada.md) (SYSTEM)
- [Grid Dependency Control (GDC)](/entities/grid-dependency-control-gdc.md) (CONCEPT)

## Relations
- drive_clean/Takeout/Drive/Optimizing CUDA Performance with FP4_FP6.docx → USES → Engineering High-Throughput Sub-Byte Quantized Inference on NVIDIA Blackwell Architectures: A Deep Dive into CUDA 13.1 C
- sm120_mma_tma_blockwise_scaling.hpp → USES → x86_64
- SM121 → USES → aarch64
- sm120_mma_tma_blockwise_scaling.hpp → USES → GDDR7
- SM121 → USES → LPDDR5x
- compute_120 → PART_OF → CUDA 13.x
- compute_120a → PART_OF → CUDA 13.x
- compute_121a → PART_OF → CUDA 13.x
- compute_120f → PART_OF → CUDA 13.x
- compute_120a → USES → sm120_mma_tma_blockwise_scaling.hpp
- compute_120a → USES → SM121
- compute_120f → USES → sm120_mma_tma_blockwise_scaling.hpp
- compute_120f → USES → SM121
- NVRTC → USES → Engineering High-Throughput Sub-Byte Quantized Inference on NVIDIA Blackwell Architectures: A Deep Dive into CUDA 13.1 C
- llama.cpp → USES → Engineering High-Throughput Sub-Byte Quantized Inference on NVIDIA Blackwell Architectures: A Deep Dive into CUDA 13.1 C
- llama.cpp → USES → cuBLAS
- TensorRT → USES → CUDA 13.0
- TensorRT → USES → GB200
- TensorRT → USES → Vision Transformer (ViT)
- FLUX → USES → Hopper architectures
- CUDA 13.1 Update 2 → USES → cuBLASLtMatmul()
- CUDA 13.1 Update 2 → USES → Ada
- CUDA 13.1 Update 2 → USES → Engineering High-Throughput Sub-Byte Quantized Inference on NVIDIA Blackwell Architectures: A Deep Dive into CUDA 13.1 C
- Programmatic Dependency Launch (PDL) → USES → Engineering High-Throughput Sub-Byte Quantized Inference on NVIDIA Blackwell Architectures: A Deep Dive into CUDA 13.1 C
- Grid Dependency Control (GDC) → USES → CUTLASS autotuner
- CUTLASS autotuner → USES → sm120_mma_tma_blockwise_scaling.hpp
- CUTLASS autotuner → USES → compute_120f
- Tensor Memory Accelerator (TMA) → USES → Engineering High-Throughput Sub-Byte Quantized Inference on NVIDIA Blackwell Architectures: A Deep Dive into CUDA 13.1 C
- Warp-Specialized MMA instructions → USES → Engineering High-Throughput Sub-Byte Quantized Inference on NVIDIA Blackwell Architectures: A Deep Dive into CUDA 13.1 C
- CuTe Domain-Specific Language (DSL) → USES → CUTLASS templates
- TensorRT → USES → SM120 Blackwell platforms
- TensorStorage → USES → drive_clean/Takeout/Drive/Optimizing CUDA Performance with FP4_FP6.docx
- Google Takeout → USES → drive_clean/Takeout/Drive/Optimizing CUDA Performance with FP4_FP6.docx
- drive_clean/Takeout/Drive/Optimizing CUDA Performance with FP4_FP6.docx → RELATED_TO → Engineering High-Throughput Sub-Byte Quantized Inference on NVIDIA Blackwell Architectures: A Deep Dive into CUDA 13.1 C
- llama.cpp → USES → Driver r577 / CUDA 12.8
- vLLM → USES → drive_clean/Takeout/Drive/Optimizing CUDA Performance with FP4_FP6.docx
- vLLM → USES → compute_120a
- vLLM → USES → compute_120f
- SGLang → USES → drive_clean/Takeout/Drive/Optimizing CUDA Performance with FP4_FP6.docx
- SGLang → USES → compute_120a
- SGLang → USES → compute_120f
- B200 → PART_OF → SM100
- sm120_mma_tma_blockwise_scaling.hpp → USES → Tensor Core formats
- SM121 → USES → Tensor Core formats
- sm120_mma_tma_blockwise_scaling.hpp → USES → Tensor Memory Accelerator (TMA)
- SM121 → USES → Tensor Memory Accelerator (TMA)
- sm120_mma_tma_blockwise_scaling.hpp → USES → thread block clusters
- SM121 → USES → thread block clusters
- CUTLASS autotuner → USES → compute_120a
- CUDA Toolkit 13.1 → USES → MSVC
- sm120_blockscaled_mma_tma.hpp → USES → cute::ArrayEngine
- sm120_mma_tma_blockwise_scaling.hpp → USES → cute::array_aligned
- smem_SFA → USES → cute::ArrayEngine
- smem_SFB → USES → cute::ArrayEngine
- Tensor Core formats → PART_OF → sm120_mma_tma_blockwise_scaling.hpp
- Tensor Core formats → PART_OF → SM121
- Tensor Memory Accelerator (TMA) → PART_OF → sm120_mma_tma_blockwise_scaling.hpp
- Tensor Memory Accelerator (TMA) → PART_OF → SM121
- thread block clusters → PART_OF → sm120_mma_tma_blockwise_scaling.hpp
- thread block clusters → PART_OF → SM121
- CUDA 13.0 → USES → compute_120f
- TensorRT → USES → CUDA 12.9
- llama.cpp → USES → RTX 5090
