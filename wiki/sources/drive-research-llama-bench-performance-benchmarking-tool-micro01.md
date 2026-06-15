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
- [[cuda|CUDA]] (SYSTEM)
- [[tesla-v100|Tesla V100]] (SYSTEM)
- [[flash-attention|Flash Attention]] (CONCEPT)
- [[nvidia-rtx-4090-d|NVIDIA RTX 4090 D]] (SYSTEM)
- [[rocm|ROCm]] (SYSTEM)
- [[m3-max|M3 Max]] (SYSTEM)
- [[gguf|GGUF]] (CONCEPT)
- [[nvidia-rtx-a2000|NVIDIA RTX A2000]] (SYSTEM)
- [[llama-cpp|llama.cpp]] (SYSTEM)
- [[nvidia-rtx-5060-ti|NVIDIA RTX 5060 Ti]] (SYSTEM)
- [[nvidia-a100-pcie|NVIDIA A100 PCIe]] (SYSTEM)
- [[m1-max|M1 Max]] (SYSTEM)
- [[ryzen-ai-7-350|Ryzen AI 7 350]] (SYSTEM)
- [[llm-inference|LLM Inference]] (CONCEPT)
- [[llama-bench|llama-bench]] (TOOL)
- [[nvidia-rtx-6000-ada|NVIDIA RTX 6000 Ada]] (SYSTEM)
- [[prompt-processing|Prompt Processing]] (CONCEPT)
- [[gtx-1080-ti|GTX 1080 Ti]] (SYSTEM)
- [[mac-studio-m1-ultra|Mac Studio M1 Ultra]] (SYSTEM)
- [[m2-ultra|M2 Ultra]] (SYSTEM)
- [[ggml|GGML]] (SYSTEM)
- [[nvidia-rtx-3090-ti|NVIDIA RTX 3090 Ti]] (SYSTEM)
- [[apple-silicon|Apple Silicon]] (SYSTEM)
- [[sycl|SYCL]] (SYSTEM)
- [[nvidia-h100-pcie|NVIDIA H100 PCIe]] (SYSTEM)
- [[nvidia-rtx-5080|NVIDIA RTX 5080]] (SYSTEM)
- [[nvidia-dgx-spark|NVIDIA DGX Spark]] (SYSTEM)
- [[amd-ai-9-hx-370|AMD AI 9 HX 370]] (SYSTEM)
- [[nvidia-pro-4000-blackwell|NVIDIA PRO 4000 Blackwell]] (SYSTEM)
- [[nvidia-rtx-5070|NVIDIA RTX 5070]] (SYSTEM)
- [[nvidia-rtx-pro-6000-blackwell|NVIDIA RTX PRO 6000 Blackwell]] (SYSTEM)
- [[vulkan|Vulkan]] (SYSTEM)
- [[nvidia-rtx-3060|NVIDIA RTX 3060]] (SYSTEM)
- [[token-generation|Token Generation]] (CONCEPT)

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
