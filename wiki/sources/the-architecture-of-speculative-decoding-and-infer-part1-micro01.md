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
- [[cpu|CPU]] (SYSTEM)
- [[llama-cpp|llama.cpp]] (TOOL)
- [[sycl|SYCL]] (TOOL)
- [[draft-model|Draft Model]] (CONCEPT)
- [[vulkan|Vulkan]] (TOOL)
- [[qwen2-5-3b-instruct|Qwen2.5-3B-Instruct]] (SYSTEM)
- [[hipblas-rocm|hipBLAS (ROCm)]] (TOOL)
- [[qwen2-5-72b|Qwen2.5-72B]] (SYSTEM)
- [[gguf|GGUF]] (CONCEPT)
- [[llama-3-1-70b|Llama 3.1 70B]] (SYSTEM)
- [[edge-hardware|Edge Hardware]] (CONCEPT)
- [[metal|Metal]] (TOOL)
- [[llama-3-2-1b|Llama 3.2 1B]] (SYSTEM)
- [[turbospec|TurboSpec]] (CONCEPT)
- [[byte-pair-encoding-bpe|Byte Pair Encoding (BPE)]] (CONCEPT)
- [[moe-i-o-prefetching-moe-speq|MoE I/O Prefetching (MoE-SpeQ)]] (CONCEPT)
- [[autoregressive-generation|Autoregressive Generation]] (CONCEPT)
- [[n-gram|N-Gram]] (CONCEPT)
- [[cuda|CUDA]] (TOOL)
- [[online-speculative-decoding-osd|Online Speculative Decoding (OSD)]] (CONCEPT)
- [[openblas|OpenBLAS]] (TOOL)
- [[qwen2-5-0-5b-instruct|Qwen2.5-0.5B-Instruct]] (SYSTEM)
- [[qwen2-5-ecosystem|Qwen2.5 Ecosystem]] (ORGANIZATION)
- [[target-model|Target Model]] (CONCEPT)
- [[vram|VRAM]] (CONCEPT)
- [[nvidia-rtx-4060|NVIDIA RTX 4060]] (SYSTEM)

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
