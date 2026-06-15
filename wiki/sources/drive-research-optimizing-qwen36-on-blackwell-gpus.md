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
- [[qwen3-6-35b-a3b|Qwen3.6-35B-A3B]] (MODEL)
- [[vllm|vLLM]] (TOOL)
- [[qwen3-6-27b|Qwen3.6-27B]] (MODEL)
- [[nvidia-modelopt|NVIDIA Modelopt]] (TOOL)
- [[quantization-aware-distillation-qad|Quantization Aware Distillation (QAD)]] (CONCEPT)
- [[ollama|Ollama]] (TOOL)
- [[pipeline-parallelism-pp|Pipeline Parallelism (PP)]] (CONCEPT)
- [[wsl2|WSL2]] (SYSTEM)
- [[mixture-of-experts-moe|Mixture of Experts (MoE)]] (CONCEPT)
- [[pcie-gen-5|PCIe Gen 5]] (CONCEPT)
- [[llm-compressor|LLM Compressor]] (TOOL)
- [[expert-parallelism-ep|Expert Parallelism (EP)]] (CONCEPT)
- [[nvlink|NVLink]] (CONCEPT)
- [[mamba2|Mamba2]] (SYSTEM)
- [[geforce-rtx-5070-ti|GeForce RTX 5070 Ti]] (SYSTEM)
- [[gated-deltanet-gdn|Gated DeltaNet (GDN)]] (CONCEPT)
- [[nvfp4-quantization|NVFP4 Quantization]] (CONCEPT)
- [[llama-cpp|llama.cpp]] (TOOL)
- [[tensor-parallelism-tp|Tensor Parallelism (TP)]] (CONCEPT)
- [[unsloth-studio|Unsloth Studio]] (TOOL)
- [[cuda-13-0|CUDA 13.0]] (SYSTEM)
- [[multi-token-prediction-mtp|Multi-Token Prediction (MTP)]] (CONCEPT)
- [[cuda-13-2|CUDA 13.2]] (SYSTEM)
- [[blackwell-gpus|Blackwell GPUs]] (SYSTEM)
- [[model-runner-v2-mrv2|Model Runner V2 (MRV2)]] (CONCEPT)

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
