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
- [mergekit-moe](/entities/mergekit-moe.md) (TOOL)
- [LaserRMT](/entities/laserrmt.md) (CONCEPT)
- [Kunoichi-DPO-v2-7B](/entities/kunoichi-dpo-v2-7b.md) (BOOK)
- [deepseek-ai/DeepSeek-R1-Distill-Qwen-7B](/entities/deepseek-ai-deepseek-r1-distill-qwen-7b.md) (ORGANIZATION)
- [Expert Parallelism (EP)](/entities/expert-parallelism-ep.md) (CONCEPT)
- [Qwen/Qwen2.5-Coder-7B-Instruct](/entities/qwen-qwen2-5-coder-7b-instruct.md) (BOOK)
- [sovereign-moe.yaml](/entities/sovereign-moe-yaml.md) (TOOL)
- [NeuralDaredevil-7B](/entities/neuraldaredevil-7b.md) (BOOK)
- [Mixture of Experts (MoE)](/entities/mixture-of-experts-moe.md) (CONCEPT)
- [vLLM](/entities/vllm.md) (SYSTEM)
- [FrankenMoE](/entities/frankenmoe.md) (CONCEPT)
- [AlphaMonarch-7B](/entities/alphamonarch-7b.md) (BOOK)
- [MC-SMoE](/entities/mc-smoe.md) (CONCEPT)
- [Tensor Parallelism (TP)](/entities/tensor-parallelism-tp.md) (CONCEPT)
- [GGUF](/entities/gguf.md) (CONCEPT)
- [Transformer](/entities/transformer.md) (CONCEPT)
- [llama.cpp](/entities/llama-cpp.md) (SYSTEM)
- [mlabonne/Beyonder-4x7B-v3](/entities/mlabonne-beyonder-4x7b-v3.md) (BOOK)
- [CodeNinja-1.0-7B](/entities/codeninja-1-0-7b.md) (BOOK)
- [MergeMoE](/entities/mergemoe.md) (CONCEPT)
- [Expert Pruning](/entities/expert-pruning.md) (CONCEPT)

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
