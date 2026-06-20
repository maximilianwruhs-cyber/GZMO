---
type: source
title: drive-research-optimizing-qwen36-on-blackwell-gpus
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-optimizing-qwen36-on-blackwell-gpus

Ingested source summary (2026-06-08).

## Entities
- [Qwen3.6-35B-A3B](/entities/qwen3-6-35b-a3b.md) (MODEL)
- [vLLM](/entities/vllm.md) (TOOL)
- [Qwen3.6-27B](/entities/qwen3-6-27b.md) (MODEL)
- [NVIDIA Modelopt](/entities/nvidia-modelopt.md) (TOOL)
- [Quantization Aware Distillation (QAD)](/entities/quantization-aware-distillation-qad.md) (CONCEPT)
- [Ollama](/entities/ollama.md) (TOOL)
- [Pipeline Parallelism (PP)](/entities/pipeline-parallelism-pp.md) (CONCEPT)
- [WSL2](/entities/wsl2.md) (SYSTEM)
- [Mixture of Experts (MoE)](/entities/mixture-of-experts-moe.md) (CONCEPT)
- [PCIe Gen 5](/entities/pcie-gen-5.md) (CONCEPT)
- [LLM Compressor](/entities/llm-compressor.md) (TOOL)
- [Expert Parallelism (EP)](/entities/expert-parallelism-ep.md) (CONCEPT)
- [NVLink](/entities/nvlink.md) (CONCEPT)
- [Mamba2](/entities/mamba2.md) (SYSTEM)
- [GeForce RTX 5070 Ti](/entities/geforce-rtx-5070-ti.md) (SYSTEM)
- [Gated DeltaNet (GDN)](/entities/gated-deltanet-gdn.md) (CONCEPT)
- [NVFP4 Quantization](/entities/nvfp4-quantization.md) (CONCEPT)
- [llama.cpp](/entities/llama-cpp.md) (TOOL)
- [Tensor Parallelism (TP)](/entities/tensor-parallelism-tp.md) (CONCEPT)
- [Unsloth Studio](/entities/unsloth-studio.md) (TOOL)
- [CUDA 13.0](/entities/cuda-13-0.md) (SYSTEM)
- [Multi-Token Prediction (MTP)](/entities/multi-token-prediction-mtp.md) (CONCEPT)
- [CUDA 13.2](/entities/cuda-13-2.md) (SYSTEM)
- [Blackwell GPUs](/entities/blackwell-gpus.md) (SYSTEM)
- [Model Runner V2 (MRV2)](/entities/model-runner-v2-mrv2.md) (CONCEPT)

## Relations
- Qwen3.6-35B-A3B → USES → Mixture of Experts (MoE)
- Qwen3.6-35B-A3B → USES → Gated DeltaNet (GDN)
- Qwen3.6-35B-A3B → USES → Blackwell GPUs
- Gated DeltaNet (GDN) → RELATED_TO → Mamba2
- vLLM → USES → Qwen3.6-35B-A3B
- vLLM → USES → GeForce RTX 5070 Ti
- GeForce RTX 5070 Ti → PART_OF → Blackwell GPUs
- Tensor Parallelism (TP) → RELATED_TO → PCIe Gen 5
- Pipeline Parallelism (PP) → RELATED_TO → PCIe Gen 5
- Expert Parallelism (EP) → RELATED_TO → PCIe Gen 5
- Tensor Parallelism (TP) → RELATED_TO → NVLink
- NVFP4 Quantization → USES → Blackwell GPUs
- LLM Compressor → USES → NVFP4 Quantization
- Quantization Aware Distillation (QAD) → RELATED_TO → NVFP4 Quantization
- Ollama → USES → Qwen3.6-35B-A3B
- vLLM → USES → WSL2
- vLLM → USES → CUDA 13.0
- llama.cpp → USES → Qwen3.6-35B-A3B
- llama.cpp → USES → Multi-Token Prediction (MTP)
- Unsloth Studio → USES → Qwen3.6-35B-A3B
- Unsloth Studio → USES → Multi-Token Prediction (MTP)
- Multi-Token Prediction (MTP) → USES → Qwen3.6-27B
- Multi-Token Prediction (MTP) → USES → Qwen3.6-35B-A3B
- NVIDIA Modelopt → USES → NVFP4 Quantization
