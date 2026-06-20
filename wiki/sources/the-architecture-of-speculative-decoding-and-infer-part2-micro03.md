---
type: source
title: the-architecture-of-speculative-decoding-and-infer-part2-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# the-architecture-of-speculative-decoding-and-infer-part2-micro03

Ingested source summary (2026-06-09).

## Entities
- [Microsoft Phi-3.5 Mini (3.8B)](/entities/microsoft-phi-3-5-mini-3-8b.md) (ORGANIZATION)
- [SLM (Reasoning)](/entities/slm-reasoning.md) (CONCEPT)
- [Meta Llama 3.2 (1B and 3B)](/entities/meta-llama-3-2-1b-and-3b.md) (SYSTEM)
- [Mistral 7B](/entities/mistral-7b.md) (SYSTEM)
- [BGE-Reranker (v2-M3)](/entities/bge-reranker-v2-m3.md) (SYSTEM)
- [Llama 3.1 8B](/entities/llama-3-1-8b.md) (SYSTEM)
- [Retrieval-Augmented Generation (RAG) pipelines](/entities/retrieval-augmented-generation-rag-pipelines.md) (CONCEPT)
- [OPT 125M](/entities/opt-125m.md) (SYSTEM)
- [Llama 3.2 3B](/entities/llama-3-2-3b.md) (SYSTEM)
- [OPT 350M](/entities/opt-350m.md) (SYSTEM)
- [OPT 65B](/entities/opt-65b.md) (SYSTEM)
- [Alibaba](/entities/alibaba.md) (ORGANIZATION)
- [Cohere Rerank 3](/entities/cohere-rerank-3.md) (SYSTEM)
- [speculative decoding](/entities/speculative-decoding.md) (CONCEPT)
- [OPT 66B](/entities/opt-66b.md) (SYSTEM)
- [Gemma 2 2B/9B](/entities/gemma-2-2b-9b.md) (SYSTEM)
- [TurboQuant KV cache compression](/entities/turboquant-kv-cache-compression.md) (TOOL)
- [Llama 3.3 70B](/entities/llama-3-3-70b.md) (SYSTEM)
- [Qwen3-Reranker Series (0.6B to 8B)](/entities/qwen3-reranker-series-0-6b-to-8b.md) (SYSTEM)
- [Phi-4](/entities/phi-4.md) (SYSTEM)
- [Google Gemma 3](/entities/google-gemma-3.md) (SYSTEM)
- [OPT 1.3B](/entities/opt-1-3b.md) (SYSTEM)
- [Micro-Model](/entities/micro-model.md) (CONCEPT)
- [Qwen 2.5 1.5B](/entities/qwen-2-5-1-5b.md) (SYSTEM)
- [Qwen 2.5 72B](/entities/qwen-2-5-72b.md) (SYSTEM)
- [Qwen 2.5 7B](/entities/qwen-2-5-7b.md) (SYSTEM)
- [Meta](/entities/meta.md) (ORGANIZATION)
- [Mixtral 8x7B (MoE)](/entities/mixtral-8x7b-moe.md) (SYSTEM)
- [BAAI](/entities/baai.md) (ORGANIZATION)
- [Llama 3.1 405B](/entities/llama-3-1-405b.md) (SYSTEM)
- [Gemini](/entities/gemini.md) (SYSTEM)
- [Qwen 2.5 (0.5B)](/entities/qwen-2-5-0-5b.md) (SYSTEM)

## Relations
- Meta Llama 3.2 (1B and 3B) → PART_OF → Meta
- Qwen3-Reranker Series (0.6B to 8B) → PART_OF → Alibaba
- BGE-Reranker (v2-M3) → PART_OF → BAAI
- Micro-Model → RELATED_TO → Qwen 2.5 0.5B
- BGE-Reranker (v2-M3) → USES → Retrieval-Augmented Generation (RAG) pipelines
- TurboQuant KV cache compression → USES → speculative decoding
- Llama 3.3 70B → USES → speculative decoding
- Meta Llama 3.2 (1B and 3B) → RELATED_TO → Llama 3.3 70B
- Llama 3.2 3B → RELATED_TO → Llama 3.3 70B
- Llama 3.1 8B → RELATED_TO → Llama 3.1 405B
- Llama 3.1 8B → RELATED_TO → Llama 3.3 70B
- Llama 3.1 405B → USES → speculative decoding
- Llama 3.3 70B → RELATED_TO → Llama 3.1 405B
- Qwen 2.5 72B → USES → speculative decoding
- Qwen 2.5 7B → RELATED_TO → Qwen 2.5 72B
- Qwen 2.5 1.5B → RELATED_TO → Qwen 2.5 72B
- OPT 66B → USES → speculative decoding
- OPT 65B → USES → speculative decoding
- OPT 125M → RELATED_TO → OPT 66B
- OPT 125M → RELATED_TO → OPT 65B
- OPT 350M → RELATED_TO → OPT 66B
- OPT 350M → RELATED_TO → OPT 65B
- OPT 1.3B → RELATED_TO → OPT 66B
- OPT 1.3B → RELATED_TO → OPT 65B
- Mixtral 8x7B (MoE) → USES → speculative decoding
- Mistral 7B → RELATED_TO → Mixtral 8x7B (MoE)
- Llama 3.1 8B → USES → speculative decoding
- Llama 3.2 3B → USES → speculative decoding
