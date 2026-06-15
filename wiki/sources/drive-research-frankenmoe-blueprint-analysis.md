---
type: source
title: drive-research-frankenmoe-blueprint-analysis
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-frankenmoe-blueprint-analysis

Ingested source summary (2026-06-08).

## Entities
- [[mergekit-moe|mergekit-moe]] (TOOL)
- [[laserrmt|LaserRMT]] (CONCEPT)
- [[kunoichi-dpo-v2-7b|Kunoichi-DPO-v2-7B]] (BOOK)
- [[deepseek-ai-deepseek-r1-distill-qwen-7b|deepseek-ai/DeepSeek-R1-Distill-Qwen-7B]] (ORGANIZATION)
- [[expert-parallelism-ep|Expert Parallelism (EP)]] (CONCEPT)
- [[qwen-qwen2-5-coder-7b-instruct|Qwen/Qwen2.5-Coder-7B-Instruct]] (BOOK)
- [[sovereign-moe-yaml|sovereign-moe.yaml]] (TOOL)
- [[neuraldaredevil-7b|NeuralDaredevil-7B]] (BOOK)
- [[mixture-of-experts-moe|Mixture of Experts (MoE)]] (CONCEPT)
- [[vllm|vLLM]] (SYSTEM)
- [[frankenmoe|FrankenMoE]] (CONCEPT)
- [[alphamonarch-7b|AlphaMonarch-7B]] (BOOK)
- [[mc-smoe|MC-SMoE]] (CONCEPT)
- [[tensor-parallelism-tp|Tensor Parallelism (TP)]] (CONCEPT)
- [[gguf|GGUF]] (CONCEPT)
- [[transformer|Transformer]] (CONCEPT)
- [[llama-cpp|llama.cpp]] (SYSTEM)
- [[mlabonne-beyonder-4x7b-v3|mlabonne/Beyonder-4x7B-v3]] (BOOK)
- [[codeninja-1-0-7b|CodeNinja-1.0-7B]] (BOOK)
- [[mergemoe|MergeMoE]] (CONCEPT)
- [[expert-pruning|Expert Pruning]] (CONCEPT)

## Relations
- FrankenMoE → RELATED_TO → Mixture of Experts (MoE)
- Qwen/Qwen2.5-Coder-7B-Instruct → PART_OF → FrankenMoE
- deepseek-ai/DeepSeek-R1-Distill-Qwen-7B → PART_OF → FrankenMoE
- mergekit-moe → USES → FrankenMoE
- Transformer → RELATED_TO → FrankenMoE
- vLLM → USES → FrankenMoE
- llama.cpp → USES → FrankenMoE
- mlabonne/Beyonder-4x7B-v3 → RELATED_TO → FrankenMoE
- AlphaMonarch-7B → PART_OF → mlabonne/Beyonder-4x7B-v3
- NeuralDaredevil-7B → PART_OF → mlabonne/Beyonder-4x7B-v3
- Kunoichi-DPO-v2-7B → PART_OF → mlabonne/Beyonder-4x7B-v3
- CodeNinja-1.0-7B → PART_OF → mlabonne/Beyonder-4x7B-v3
- Expert Pruning → RELATED_TO → FrankenMoE
- MC-SMoE → RELATED_TO → FrankenMoE
- MergeMoE → RELATED_TO → FrankenMoE
- LaserRMT → RELATED_TO → FrankenMoE
- FrankenMoE → USES → mergekit-moe
- mergekit-moe → USES → sovereign-moe.yaml
- vLLM → USES → Expert Parallelism (EP)
- Expert Parallelism (EP) → RELATED_TO → Tensor Parallelism (TP)
- llama.cpp → USES → GGUF
- Expert Pruning → RELATED_TO → Mixture of Experts (MoE)
- MC-SMoE → RELATED_TO → Mixture of Experts (MoE)
- MergeMoE → RELATED_TO → Mixture of Experts (MoE)
- LaserRMT → RELATED_TO → Mixture of Experts (MoE)
