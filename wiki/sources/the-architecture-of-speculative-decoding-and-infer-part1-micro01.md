---
type: source
title: the-architecture-of-speculative-decoding-and-infer-part1-micro01
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# the-architecture-of-speculative-decoding-and-infer-part1-micro01

Ingested source summary (2026-06-09).

## Entities
- [CPU](/entities/cpu.md) (SYSTEM)
- [llama.cpp](/entities/llama-cpp.md) (TOOL)
- [SYCL](/entities/sycl.md) (TOOL)
- [Draft Model](/entities/draft-model.md) (CONCEPT)
- [Vulkan](/entities/vulkan.md) (TOOL)
- [Qwen2.5-3B-Instruct](/entities/qwen2-5-3b-instruct.md) (SYSTEM)
- [hipBLAS (ROCm)](/entities/hipblas-rocm.md) (TOOL)
- [Qwen2.5-72B](/entities/qwen2-5-72b.md) (SYSTEM)
- [GGUF](/entities/gguf.md) (CONCEPT)
- [Llama 3.1 70B](/entities/llama-3-1-70b.md) (SYSTEM)
- [Edge Hardware](/entities/edge-hardware.md) (CONCEPT)
- [Metal](/entities/metal.md) (TOOL)
- [Llama 3.2 1B](/entities/llama-3-2-1b.md) (SYSTEM)
- [TurboSpec](/entities/turbospec.md) (CONCEPT)
- [Byte Pair Encoding (BPE)](/entities/byte-pair-encoding-bpe.md) (CONCEPT)
- [MoE I/O Prefetching (MoE-SpeQ)](/entities/moe-i-o-prefetching-moe-speq.md) (CONCEPT)
- [Autoregressive Generation](/entities/autoregressive-generation.md) (CONCEPT)
- [N-Gram](/entities/n-gram.md) (CONCEPT)
- [CUDA](/entities/cuda.md) (TOOL)
- [Online Speculative Decoding (OSD)](/entities/online-speculative-decoding-osd.md) (CONCEPT)
- [OpenBLAS](/entities/openblas.md) (TOOL)
- [Qwen2.5-0.5B-Instruct](/entities/qwen2-5-0-5b-instruct.md) (SYSTEM)
- [Qwen2.5 Ecosystem](/entities/qwen2-5-ecosystem.md) (ORGANIZATION)
- [Target Model](/entities/target-model.md) (CONCEPT)
- [VRAM](/entities/vram.md) (CONCEPT)
- [NVIDIA RTX 4060](/entities/nvidia-rtx-4060.md) (SYSTEM)

## Relations
- Online Speculative Decoding (OSD) → RELATED_TO → Autoregressive Generation
- Online Speculative Decoding (OSD) → USES → Draft Model
- Online Speculative Decoding (OSD) → USES → Target Model
- llama.cpp → USES → Online Speculative Decoding (OSD)
- llama.cpp → USES → N-Gram
- Draft Model → RELATED_TO → Target Model
- Qwen2.5 Ecosystem → PART_OF → Qwen2.5-0.5B-Instruct
- Qwen2.5 Ecosystem → PART_OF → Qwen2.5-3B-Instruct
- Qwen2.5-0.5B-Instruct → RELATED_TO → Draft Model
- Qwen2.5-3B-Instruct → RELATED_TO → Target Model
- Qwen2.5-0.5B-Instruct → USES → Byte Pair Encoding (BPE)
- Qwen2.5-3B-Instruct → USES → Byte Pair Encoding (BPE)
- Edge Hardware → RELATED_TO → VRAM
- NVIDIA RTX 4060 → PART_OF → VRAM
- llama.cpp → USES → CUDA
- llama.cpp → USES → Metal
- llama.cpp → USES → SYCL
- llama.cpp → USES → Vulkan
- llama.cpp → USES → hipBLAS (ROCm)
- llama.cpp → USES → OpenBLAS
- Qwen2.5-72B → RELATED_TO → Qwen2.5-3B-Instruct
- Llama 3.1 70B → RELATED_TO → Llama 3.2 1B
- TurboSpec → RELATED_TO → Online Speculative Decoding (OSD)
- MoE I/O Prefetching (MoE-SpeQ) → RELATED_TO → Online Speculative Decoding (OSD)
- N-Gram → RELATED_TO → Online Speculative Decoding (OSD)
