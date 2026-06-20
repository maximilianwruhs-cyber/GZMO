---
type: source
title: drive-research-llama-bench-performance-benchmarking-tool-micro01
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-llama-bench-performance-benchmarking-tool-micro01

Ingested source summary (2026-06-09).

## Entities
- [CUDA](/entities/cuda.md) (SYSTEM)
- [Tesla V100](/entities/tesla-v100.md) (SYSTEM)
- [Flash Attention](/entities/flash-attention.md) (CONCEPT)
- [NVIDIA RTX 4090 D](/entities/nvidia-rtx-4090-d.md) (SYSTEM)
- [ROCm](/entities/rocm.md) (SYSTEM)
- [M3 Max](/entities/m3-max.md) (SYSTEM)
- [GGUF](/entities/gguf.md) (CONCEPT)
- [NVIDIA RTX A2000](/entities/nvidia-rtx-a2000.md) (SYSTEM)
- [llama.cpp](/entities/llama-cpp.md) (SYSTEM)
- [NVIDIA RTX 5060 Ti](/entities/nvidia-rtx-5060-ti.md) (SYSTEM)
- [NVIDIA A100 PCIe](/entities/nvidia-a100-pcie.md) (SYSTEM)
- [M1 Max](/entities/m1-max.md) (SYSTEM)
- [Ryzen AI 7 350](/entities/ryzen-ai-7-350.md) (SYSTEM)
- [LLM Inference](/entities/llm-inference.md) (CONCEPT)
- [llama-bench](/entities/llama-bench.md) (TOOL)
- [NVIDIA RTX 6000 Ada](/entities/nvidia-rtx-6000-ada.md) (SYSTEM)
- [Prompt Processing](/entities/prompt-processing.md) (CONCEPT)
- [GTX 1080 Ti](/entities/gtx-1080-ti.md) (SYSTEM)
- [Mac Studio M1 Ultra](/entities/mac-studio-m1-ultra.md) (SYSTEM)
- [M2 Ultra](/entities/m2-ultra.md) (SYSTEM)
- [GGML](/entities/ggml.md) (SYSTEM)
- [NVIDIA RTX 3090 Ti](/entities/nvidia-rtx-3090-ti.md) (SYSTEM)
- [Apple Silicon](/entities/apple-silicon.md) (SYSTEM)
- [SYCL](/entities/sycl.md) (SYSTEM)
- [NVIDIA H100 PCIe](/entities/nvidia-h100-pcie.md) (SYSTEM)
- [NVIDIA RTX 5080](/entities/nvidia-rtx-5080.md) (SYSTEM)
- [NVIDIA DGX Spark](/entities/nvidia-dgx-spark.md) (SYSTEM)
- [AMD AI 9 HX 370](/entities/amd-ai-9-hx-370.md) (SYSTEM)
- [NVIDIA PRO 4000 Blackwell](/entities/nvidia-pro-4000-blackwell.md) (SYSTEM)
- [NVIDIA RTX 5070](/entities/nvidia-rtx-5070.md) (SYSTEM)
- [NVIDIA RTX PRO 6000 Blackwell](/entities/nvidia-rtx-pro-6000-blackwell.md) (SYSTEM)
- [Vulkan](/entities/vulkan.md) (SYSTEM)
- [NVIDIA RTX 3060](/entities/nvidia-rtx-3060.md) (SYSTEM)
- [Token Generation](/entities/token-generation.md) (CONCEPT)

## Relations
- llama-bench → PART_OF → llama.cpp
- llama-bench → USES → GGML
- Prompt Processing → PART_OF → GGML
- Token Generation → PART_OF → GGML
- llama-bench → USES → Prompt Processing
- llama-bench → USES → Token Generation
- llama-bench → USES → LLM Inference
- llama-bench → USES → GGUF
- CUDA → RELATED_TO → NVIDIA RTX 4090 D
- CUDA → RELATED_TO → NVIDIA RTX 3090 Ti
- ROCm → RELATED_TO → AMD AI 9 HX 370
- ROCm → RELATED_TO → Ryzen AI 7 350
- Vulkan → RELATED_TO → AMD AI 9 HX 370
- Vulkan → RELATED_TO → Ryzen AI 7 350
- Flash Attention → RELATED_TO → ROCm
- Flash Attention → RELATED_TO → Vulkan
- CUDA → RELATED_TO → NVIDIA DGX Spark
- CUDA → RELATED_TO → NVIDIA RTX A2000
