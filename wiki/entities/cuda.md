---
type: entity
title: CUDA
created: 2026-06-08
updated: 2026-06-10
sources: 18
tags: []
status: draft
gzmo_synthetic: true
---



















# CUDA

Type: TOOL

## From [drive-research-blackwell-sm120-gemm-optimization-guide](/entities/drive-research-blackwell-sm120-gemm-optimization-guide.md) (2026-06-08)
- CUDA 13.0 introduces three distinct targets for SM12x generation.
- CUDA 13.1 is recommended for peak performance path.
- CUDA 13.0
- CUDA Programming Guide
- CUDA Programming and Performance

## From [drive-research-recursivemas-add-info](/entities/drive-research-recursivemas-add-info.md) (2026-06-08)
- Required for local execution of the system.
- Tensors in the latent space are processed on the graphics card.

## From [drive-research-to-product-engineering-leadership](/entities/drive-research-to-product-engineering-leadership.md) (2026-06-08)
- NVIDIA technology for GPU computing.
- Requires safe dynamic loading to check for presence.

## From [architectures-and-optimizations-for-speculative-de-micro03](/entities/architectures-and-optimizations-for-speculative-de-micro03.md) (2026-06-09)
- A CUDA-capable graphics card (GPU) is required to run the system locally.
- Tensors in the latent space need to be processed on the graphics card.

## From [drive-research-32gb-vram-ai-reasoning-models-micro01](/entities/drive-research-32gb-vram-ai-reasoning-models-micro01.md) (2026-06-09)
- Architecture for 32 GB VRAM environments
- Used in conjunction with NVIDIA hardware

## From [drive-research-32gb-vram-ai-reasoning-models-micro02](/entities/drive-research-32gb-vram-ai-reasoning-models-micro02.md) (2026-06-09)
- TensorRT-LLM is fundamentally coupled to the underlying CUDA architecture.

## From [drive-research-llama-bench-performance-benchmarking-tool-micro01](/entities/drive-research-llama-bench-performance-benchmarking-tool-micro01.md) (2026-06-09)
- Backend for high-performance LLM execution.
- Scaling to multi-GPU configurations introduces challenges.
- Scoreboards provided for NVIDIA architectures running CUDA.

## From [drive-research-llamacpp-optimization-blueprint-micro03](/entities/drive-research-llamacpp-optimization-blueprint-micro03.md) (2026-06-09)
- CUDA cores' arithmetic intensity is maximized by high logical batch size.
- TurboQuant algorithms are integrated into the llama.cpp CUDA backend.

## From [drive-research-llm-inference-engine-audit-2026-micro02](/entities/drive-research-llm-inference-engine-audit-2026-micro02.md) (2026-06-09)
- Used by llama.cpp on NVIDIA graphics cards.

## From [drive-research-marlin-baseline-for-early-deployments-micro02](/entities/drive-research-marlin-baseline-for-early-deployments-micro02.md) (2026-06-09)
- Graph capture can trigger illegal memory access crashes.
- Version 13.0+ is recommended for native FP4 execution.

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro01](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro01.md) (2026-06-09)
- A compute backend for NVIDIA GPUs.
- Used when compiling llama.cpp for NVIDIA GPUs.

## From [optimizing-nvidia-blackwell-sm120-part1-micro01](/entities/optimizing-nvidia-blackwell-sm120-part1-micro01.md) (2026-06-10)
- A compute architecture used for GPU-accelerated inference.
- Requires the -DGGML_CUDA=ON flag for compilation in llama.cpp.
- Utilizes custom kernels to bypass standard libraries like cuBLAS for quantized models.

## From [optimizing-nvidia-blackwell-sm120-part1-micro05](/entities/optimizing-nvidia-blackwell-sm120-part1-micro05.md) (2026-06-10)
- Requires version 13.0+ for native FP4 research

## From [optimizing-nvidia-blackwell-sm120-part2-micro02](/entities/optimizing-nvidia-blackwell-sm120-part2-micro02.md) (2026-06-10)
- Version 13.0 introduced new target suffixes for SM12x.

## From [optimizing-nvidia-blackwell-sm120-part2-micro04](/entities/optimizing-nvidia-blackwell-sm120-part2-micro04.md) (2026-06-10)
- Baseline backend for high-performance LLM execution.
- Used by NVIDIA architectures.

## From [optimizing-nvidia-blackwell-sm120-part3-micro03](/entities/optimizing-nvidia-blackwell-sm120-part3-micro03.md) (2026-06-10)
- An active backend queried by the engine to determine memory capacity.
- Device enumeration and primary context initialization allocate a base memory footprint of 300 to 500 MiB of VRAM per physical GPU.

## From [the-2026-linux-workstation-micro02](/entities/the-2026-linux-workstation-micro02.md) (2026-06-10)
- Parallel computing and deep learning architecture launched in 2007
- CUDA 13.0 introduced tile-based programming

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro03](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro03.md) (2026-06-10)
- Underlying architecture for TensorRT-LLM.
