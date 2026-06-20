---
type: source
title: drive-research-small-models
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-small-models

Ingested source summary (2026-06-08).

## Entities
- [Cohere Rerank 3](/entities/cohere-rerank-3.md) (SYSTEM)
- [Alibaba](/entities/alibaba.md) (ORGANIZATION)
- [BGE-Reranker (v2-M3)](/entities/bge-reranker-v2-m3.md) (SYSTEM)
- [BAAI](/entities/baai.md) (ORGANIZATION)
- [Google Gemma 3 (or Gemma 2 2B/9B)](/entities/google-gemma-3-or-gemma-2-2b-9b.md) (ORGANIZATION)
- [Microsoft Phi-3.5 Mini (3.8B) or Phi-4](/entities/microsoft-phi-3-5-mini-3-8b-or-phi-4.md) (SYSTEM)
- [Meta Llama 3.2 (1B and 3B)](/entities/meta-llama-3-2-1b-and-3b.md) (ORGANIZATION)
- [Query Rewriting](/entities/query-rewriting.md) (CONCEPT)
- [Ultra-Fast Data Parsing](/entities/ultra-fast-data-parsing.md) (CONCEPT)
- [Context Reranking](/entities/context-reranking.md) (CONCEPT)
- [Qwen3-Reranker Series (0.6B to 8B)](/entities/qwen3-reranker-series-0-6b-to-8b.md) (SYSTEM)
- [Qwen 2.5 (0.5B)](/entities/qwen-2-5-0-5b.md) (SYSTEM)
- [Retrieval-Augmented Generation (RAG) pipelines](/entities/retrieval-augmented-generation-rag-pipelines.md) (CONCEPT)
- [Gemini](/entities/gemini.md) (SYSTEM)
- [Output Verification](/entities/output-verification.md) (CONCEPT)

## Relations
- Microsoft Phi-3.5 Mini (3.8B) or Phi-4 → USES → Query Rewriting
- Microsoft Phi-3.5 Mini (3.8B) or Phi-4 → USES → Output Verification
- Google Gemma 3 (or Gemma 2 2B/9B) → USES → Query Rewriting
- Google Gemma 3 (or Gemma 2 2B/9B) → USES → Output Verification
- Meta Llama 3.2 (1B and 3B) → USES → Query Rewriting
- Meta Llama 3.2 (1B and 3B) → USES → Output Verification
- Qwen3-Reranker Series (0.6B to 8B) → USES → Context Reranking
- BGE-Reranker (v2-M3) → USES → Context Reranking
- Cohere Rerank 3 → USES → Context Reranking
- Qwen 2.5 (0.5B) → USES → Ultra-Fast Data Parsing
- Alibaba → PART_OF → Qwen3-Reranker Series (0.6B to 8B)
- BAAI → PART_OF → BGE-Reranker (v2-M3)
- BGE-Reranker (v2-M3) → USES → Retrieval-Augmented Generation (RAG) pipelines
- Gemini → RELATED_TO → Google Gemma 3 (or Gemma 2 2B/9B)
