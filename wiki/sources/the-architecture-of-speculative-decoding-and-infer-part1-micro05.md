---
type: source
title: the-architecture-of-speculative-decoding-and-infer-part1-micro05
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# the-architecture-of-speculative-decoding-and-infer-part1-micro05

Ingested source summary (2026-06-09).

## Entities
- [Speculative Decoding](/entities/speculative-decoding.md) (CONCEPT)
- [llama.cpp](/entities/llama-cpp.md) (TOOL)
- [Medusa](/entities/medusa.md) (SYSTEM)
- [PolarQuant](/entities/polarquant.md) (TOOL)
- [Large Language Models (LLMs)](/entities/large-language-models-llms.md) (CONCEPT)
- [KV cache](/entities/kv-cache.md) (CONCEPT)
- [Quantized Johnson-Lindenstrauss (QJL)](/entities/quantized-johnson-lindenstrauss-qjl.md) (TOOL)
- [ML-SpecQD](/entities/ml-specqd.md) (CONCEPT)
- [QuantSpec](/entities/quantspec.md) (CONCEPT)
- [EAGLE](/entities/eagle.md) (SYSTEM)
- [Qwen3.5-35B-A3B](/entities/qwen3-5-35b-a3b.md) (SYSTEM)
- [TurboQuant](/entities/turboquant.md) (TOOL)
- [Hugging Face](/entities/hugging-face.md) (TOOL)
- [vLLM](/entities/vllm.md) (TOOL)

## Relations
- TurboQuant → RELATED_TO → Speculative Decoding
- Speculative Decoding → USES → Large Language Models (LLMs)
- Hugging Face → USES → TurboQuant
- vLLM → USES → TurboQuant
- llama.cpp → USES → TurboQuant
- EAGLE → RELATED_TO → Speculative Decoding
- Medusa → RELATED_TO → Speculative Decoding
- TurboQuant → USES → PolarQuant
- TurboQuant → USES → Quantized Johnson-Lindenstrauss (QJL)
- ML-SpecQD → RELATED_TO → Speculative Decoding
- QuantSpec → RELATED_TO → Speculative Decoding
- Qwen3.5-35B-A3B → USES → TurboQuant
