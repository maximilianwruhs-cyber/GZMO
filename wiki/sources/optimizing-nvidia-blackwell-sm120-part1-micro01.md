---
type: source
title: optimizing-nvidia-blackwell-sm120-part1-micro01
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# optimizing-nvidia-blackwell-sm120-part1-micro01

Ingested source summary (2026-06-10).

## Entities
- [numactl](/entities/numactl.md) (TOOL)
- [Flash Attention](/entities/flash-attention.md) (CONCEPT)
- [RTX 4090](/entities/rtx-4090.md) (CONCEPT)
- [CUDA](/entities/cuda.md) (SYSTEM)
- [llama.cpp](/entities/llama-cpp.md) (SYSTEM)
- [cuBLAS](/entities/cublas.md) (TOOL)
- [CMake](/entities/cmake.md) (TOOL)
- [Blackwell](/entities/blackwell.md) (CONCEPT)
- [ggml](/entities/ggml.md) (SYSTEM)
- [RTX 3060](/entities/rtx-3060.md) (CONCEPT)
- [V100](/entities/v100.md) (CONCEPT)
- [Neoverse N2](/entities/neoverse-n2.md) (CONCEPT)
- [RDNA3+](/entities/rdna3.md) (CONCEPT)
- [RTX 5070 Ti](/entities/rtx-5070-ti.md) (CONCEPT)
- [AVX-512](/entities/avx-512.md) (CONCEPT)
- [NVCC](/entities/nvcc.md) (TOOL)
- [ROCm](/entities/rocm.md) (SYSTEM)
- [NVLink](/entities/nvlink.md) (TOOL)
- [Ryzen 9000 series](/entities/ryzen-9000-series.md) (CONCEPT)
- [CDNA](/entities/cdna.md) (CONCEPT)

## Relations
- llama.cpp → USES → ggml
- llama.cpp → USES → CUDA
- llama.cpp → USES → ROCm
- llama.cpp → USES → CMake
- llama.cpp → USES → numactl
- llama.cpp → USES → cuBLAS
