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
- [[ultra-path-interconnect-upi|Ultra Path Interconnect (UPI)]] (CONCEPT)
- [[nvidia-cuda|NVIDIA CUDA]] (SYSTEM)
- [[flash-attention-fa|Flash Attention (-fa)]] (CONCEPT)
- [[rtx-4090|RTX 4090]] (SYSTEM)
- [[rtx-3060|RTX 3060]] (SYSTEM)
- [[neoverse-n2|Neoverse N2]] (SYSTEM)
- [[retrieval-augmented-generation-rag|Retrieval-Augmented Generation (RAG)]] (CONCEPT)
- [[vram|VRAM]] (CONCEPT)
- [[drive-research-llamacpp-optimization-blueprint|drive-research-llamacpp-optimization-blueprint]] (PROJECT)
- [[quickpath-interconnect-qpi|QuickPath Interconnect (QPI)]] (CONCEPT)
- [[cmake|CMake]] (TOOL)
- [[rtx-5070-tis|RTX 5070 Tis]] (SYSTEM)
- [[drive-clean-takeout-drive-llama-cpp-optimization-blueprint-docx|drive_clean/Takeout/Drive/Llama.cpp Optimization Blueprint.docx]] (SYSTEM)
- [[cublas|cuBLAS]] (TOOL)
- [[high-bandwidth-memory-hbm|High-Bandwidth Memory (HBM)]] (CONCEPT)
- [[nvcc-compiler|NVCC compiler]] (TOOL)
- [[ryzen-9000-series|Ryzen 9000 series]] (SYSTEM)
- [[vector-neural-network-instructions-vnni|Vector Neural Network Instructions (VNNI)]] (CONCEPT)
- [[pcie|PCIe]] (CONCEPT)
- [[ggml|ggml]] (TOOL)
- [[mixture-of-experts-moe|Mixture of Experts (MoE)]] (CONCEPT)
- [[rocm|ROCm]] (SYSTEM)
- [[amd-epyc|AMD EPYC]] (SYSTEM)
- [[advanced-vector-extensions-512-avx-512|Advanced Vector Extensions 512 (AVX-512)]] (CONCEPT)
- [[intel-xeon|Intel Xeon]] (SYSTEM)
- [[blackwell-class-gpus|Blackwell-class GPUs]] (SYSTEM)
- [[cpu|CPU]] (SYSTEM)
- [[numa|NUMA]] (CONCEPT)
- [[mxfp4|MXFP4]] (CONCEPT)
- [[key-value-kv-cache|Key-Value (KV) cache]] (CONCEPT)
- [[intel-scalable-processors|Intel Scalable processors]] (SYSTEM)
- [[python|Python]] (SYSTEM)

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
