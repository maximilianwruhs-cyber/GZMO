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
- [[drive-clean-takeout-drive-optimizing-cuda-performance-with-fp4-fp6-docx|drive_clean/Takeout/Drive/Optimizing CUDA Performance with FP4_FP6.docx]] (BOOK)
- [[cutlass-autotuner|CUTLASS autotuner]] (TOOL)
- [[engineering-high-throughput-sub-byte-quantized-inference-on-nvidia-blackwell-architectures-a-deep-dive-into-cuda-13-1-c|Engineering High-Throughput Sub-Byte Quantized Inference on NVIDIA Blackwell Architectures: A Deep Dive into CUDA 13.1 C]] (BOOK)
- [[sm120-mma-tma-blockwise-scaling-hpp|sm120_mma_tma_blockwise_scaling.hpp]] (BOOK)
- [[cublasltmatmul|cuBLASLtMatmul()]] (CONCEPT)
- [[b200|B200]] (SYSTEM)
- [[nvidia-blackwell-gpu-architecture|NVIDIA Blackwell GPU architecture]] (SYSTEM)
- [[ldmatrix-sync-aligned|ldmatrix.sync.aligned]] (CONCEPT)
- [[cute-array-aligned|cute::array_aligned]] (SYSTEM)
- [[int8|INT8]] (CONCEPT)
- [[cuda-12-8|CUDA 12.8]] (SYSTEM)
- [[cute-domain-specific-language-dsl|CuTe Domain-Specific Language (DSL)]] (TOOL)
- [[smem-sfb|smem_SFB]] (SYSTEM)
- [[gb200|GB200]] (SYSTEM)
- [[wave-04-drive-research|wave_04_drive_research]] (CONCEPT)
- [[smem-b|smem_B]] (SYSTEM)
- [[lpddr5x|LPDDR5x]] (SYSTEM)
- [[sglang|SGLang]] (SYSTEM)
- [[programmatic-dependency-launch-pdl|Programmatic Dependency Launch (PDL)]] (CONCEPT)
- [[smem-a|smem_A]] (SYSTEM)
- [[google-takeout|Google Takeout]] (TOOL)
- [[sm121|SM121]] (SYSTEM)
- [[cublas|cuBLAS]] (SYSTEM)
- [[smem-sfa|smem_SFA]] (SYSTEM)
- [[tf32|TF32]] (CONCEPT)
- [[compute-120f|compute_120f]] (CONCEPT)
- [[nvrtc|NVRTC]] (TOOL)
- [[cuda-13-x|CUDA 13.x]] (SYSTEM)
- [[tensorrt|TensorRT]] (SYSTEM)
- [[vision-transformer-vit|Vision Transformer (ViT)]] (CONCEPT)
- [[ptx|PTX]] (SYSTEM)
- [[tensorstorage|TensorStorage]] (SYSTEM)
- [[sm120-blackwell-platforms|SM120 Blackwell platforms]] (SYSTEM)
- [[cute-arrayengine|cute::ArrayEngine]] (SYSTEM)
- [[compute-120a|compute_120a]] (CONCEPT)
- [[msvc|MSVC]] (TOOL)
- [[cutlass-templates|CUTLASS templates]] (SYSTEM)
- [[cuda-13-1-update-2|CUDA 13.1 Update 2]] (SYSTEM)
- [[hopper-architectures|Hopper architectures]] (SYSTEM)
- [[sm120-blockscaled-mma-tma-hpp|sm120_blockscaled_mma_tma.hpp]] (BOOK)
- [[flux|FLUX]] (PROJECT)
- [[cuda-toolkit-13-1|CUDA Toolkit 13.1]] (SYSTEM)
- [[rtx-5090|RTX 5090]] (SYSTEM)
- [[x86-64|x86_64]] (SYSTEM)
- [[tensor-memory-accelerator-tma|Tensor Memory Accelerator (TMA)]] (SYSTEM)
- [[gddr7|GDDR7]] (SYSTEM)
- [[llama-cpp|llama.cpp]] (SYSTEM)
- [[compute-121a|compute_121a]] (CONCEPT)
- [[thread-block-clusters|thread block clusters]] (CONCEPT)
- [[tensor-core-formats|Tensor Core formats]] (CONCEPT)
- [[b100|B100]] (SYSTEM)
- [[cuda-13-0|CUDA 13.0]] (SYSTEM)
- [[dgx-spark|DGX Spark]] (SYSTEM)
- [[fp16|FP16]] (CONCEPT)
- [[warp-specialized-mma-instructions|Warp-Specialized MMA instructions]] (CONCEPT)
- [[compute-120|compute_120]] (CONCEPT)
- [[sm100|SM100]] (SYSTEM)
- [[aarch64|aarch64]] (SYSTEM)
- [[cuda-12-9|CUDA 12.9]] (SYSTEM)
- [[partition-s|partition_S()]] (CONCEPT)
- [[fp8|FP8]] (CONCEPT)
- [[rtx-pro-6000|RTX PRO 6000]] (SYSTEM)
- [[bf16|BF16]] (CONCEPT)
- [[vllm|vLLM]] (SYSTEM)
- [[ada|Ada]] (SYSTEM)
- [[grid-dependency-control-gdc|Grid Dependency Control (GDC)]] (CONCEPT)

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
