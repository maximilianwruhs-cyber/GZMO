---
type: entity
title: cuBLAS
created: 2026-06-09
updated: 2026-06-10
sources: 6
tags: []
status: draft
gzmo_synthetic: true
---






# cuBLAS

Type: TOOL

## From [[drive-research-cuda-graph-capture-failure-workarounds-micro02|drive-research-cuda-graph-capture-failure-workarounds-micro02]] (2026-06-09)
- Internal workspace caching causes host-side memory leaks.
- Allocates host memory for unique matrix multiplication configurations.
- Host memory is only released when the cuBLAS handle is completely destroyed.

## From [[drive-research-llamacpp-optimization-blueprint-micro02|drive-research-llamacpp-optimization-blueprint-micro02]] (2026-06-09)
- The ggml backend deliberately bypasses standard generic libraries like this for quantized models when appropriate.
- Forcing the engine back to cuBLAS via -DGGML_CUDA_FORCE_CUBLAS=ON can increase prompt processing speeds on modern datacenter GPUs.

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- Standard routines used as a fallback when the MMQ kernel in llama.cpp crashes.
- Execution via cuBLAS incurs a severe performance penalty due to dequantization.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro05|optimizing-nvidia-blackwell-sm120-part3-micro05]] (2026-06-09)
- Internal workspace caching causes host-side memory leaks with Gemma 4.
- Allocates host memory for unique matrix multiplication configurations.
- Host memory is released when the cuBLAS handle is destroyed.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro01|optimizing-nvidia-blackwell-sm120-part1-micro01]] (2026-06-10)
- A standard generic library for matrix multiplication.
- Can be forced via -DGGML_CUDA_FORCE_CUBLAS=ON to increase prompt processing speeds.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro06|optimizing-nvidia-blackwell-sm120-part1-micro06]] (2026-06-10)
- Standard runtime library used as a fallback when MMQ kernels crash.
- Performance penalty occurs when dequantizing sub-byte weights to FP16.
