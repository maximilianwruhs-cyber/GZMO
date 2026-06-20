---
type: source
title: drive-research-llamacpp-optimization-blueprint-micro02
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-llamacpp-optimization-blueprint-micro02

Ingested source summary (2026-06-09).

## Entities
- [Ultra Path Interconnect (UPI)](/entities/ultra-path-interconnect-upi.md) (CONCEPT)
- [NVIDIA CUDA](/entities/nvidia-cuda.md) (SYSTEM)
- [Flash Attention (-fa)](/entities/flash-attention-fa.md) (CONCEPT)
- [RTX 4090](/entities/rtx-4090.md) (SYSTEM)
- [RTX 3060](/entities/rtx-3060.md) (SYSTEM)
- [Neoverse N2](/entities/neoverse-n2.md) (SYSTEM)
- [Retrieval-Augmented Generation (RAG)](/entities/retrieval-augmented-generation-rag.md) (CONCEPT)
- [VRAM](/entities/vram.md) (CONCEPT)
- [drive-research-llamacpp-optimization-blueprint](/entities/drive-research-llamacpp-optimization-blueprint.md) (PROJECT)
- [QuickPath Interconnect (QPI)](/entities/quickpath-interconnect-qpi.md) (CONCEPT)
- [CMake](/entities/cmake.md) (TOOL)
- [RTX 5070 Tis](/entities/rtx-5070-tis.md) (SYSTEM)
- [drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx](/entities/drive-clean-takeout-drive-llama-cpp-optimization-blueprint-docx.md) (SYSTEM)
- [cuBLAS](/entities/cublas.md) (TOOL)
- [High-Bandwidth Memory (HBM)](/entities/high-bandwidth-memory-hbm.md) (CONCEPT)
- [NVCC compiler](/entities/nvcc-compiler.md) (TOOL)
- [Ryzen 9000 series](/entities/ryzen-9000-series.md) (SYSTEM)
- [Vector Neural Network Instructions (VNNI)](/entities/vector-neural-network-instructions-vnni.md) (CONCEPT)
- [PCIe](/entities/pcie.md) (CONCEPT)
- [ggml](/entities/ggml.md) (TOOL)
- [Mixture of Experts (MoE)](/entities/mixture-of-experts-moe.md) (CONCEPT)
- [ROCm](/entities/rocm.md) (SYSTEM)
- [AMD EPYC](/entities/amd-epyc.md) (SYSTEM)
- [Advanced Vector Extensions 512 (AVX-512)](/entities/advanced-vector-extensions-512-avx-512.md) (CONCEPT)
- [Intel Xeon](/entities/intel-xeon.md) (SYSTEM)
- [Blackwell-class GPUs](/entities/blackwell-class-gpus.md) (SYSTEM)
- [CPU](/entities/cpu.md) (SYSTEM)
- [NUMA](/entities/numa.md) (CONCEPT)
- [MXFP4](/entities/mxfp4.md) (CONCEPT)
- [Key-Value (KV) cache](/entities/key-value-kv-cache.md) (CONCEPT)
- [Intel Scalable processors](/entities/intel-scalable-processors.md) (SYSTEM)
- [Python](/entities/python.md) (SYSTEM)

## Relations
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → RELATED_TO → Python
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → NVIDIA CUDA
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → ROCm
- ggml → USES → NVIDIA CUDA
- CMake → USES → drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx
- CMake → USES → ggml
- Advanced Vector Extensions 512 (AVX-512) → RELATED_TO → Ryzen 9000 series
- Advanced Vector Extensions 512 (AVX-512) → RELATED_TO → Intel Scalable processors
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → Advanced Vector Extensions 512 (AVX-512)
- NVIDIA CUDA → RELATED_TO → Blackwell-class GPUs
- MXFP4 → RELATED_TO → Blackwell-class GPUs
- NVCC compiler → USES → NVIDIA CUDA
- ggml → RELATED_TO → cuBLAS
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → cuBLAS
- Flash Attention (-fa) → USES → NVIDIA CUDA
- Flash Attention (-fa) → RELATED_TO → Key-Value (KV) cache
- Flash Attention (-fa) → RELATED_TO → High-Bandwidth Memory (HBM)
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → VRAM
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → PCIe
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → RTX 4090
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → Mixture of Experts (MoE)
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → CPU
- CPU → RELATED_TO → NUMA
- NUMA → RELATED_TO → Intel Xeon
- NUMA → RELATED_TO → AMD EPYC
- NUMA → RELATED_TO → QuickPath Interconnect (QPI)
- NUMA → RELATED_TO → Ultra Path Interconnect (UPI)
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → NUMA
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → Neoverse N2
- drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx → USES → Retrieval-Augmented Generation (RAG)
- drive-research-llamacpp-optimization-blueprint → RELATED_TO → drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx
