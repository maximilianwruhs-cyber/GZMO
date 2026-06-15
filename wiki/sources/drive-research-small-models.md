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
- [[cohere-rerank-3|Cohere Rerank 3]] (SYSTEM)
- [[alibaba|Alibaba]] (ORGANIZATION)
- [[bge-reranker-v2-m3|BGE-Reranker (v2-M3)]] (SYSTEM)
- [[baai|BAAI]] (ORGANIZATION)
- [[google-gemma-3-or-gemma-2-2b-9b|Google Gemma 3 (or Gemma 2 2B/9B)]] (ORGANIZATION)
- [[microsoft-phi-3-5-mini-3-8b-or-phi-4|Microsoft Phi-3.5 Mini (3.8B) or Phi-4]] (SYSTEM)
- [[meta-llama-3-2-1b-and-3b|Meta Llama 3.2 (1B and 3B)]] (ORGANIZATION)
- [[query-rewriting|Query Rewriting]] (CONCEPT)
- [[ultra-fast-data-parsing|Ultra-Fast Data Parsing]] (CONCEPT)
- [[context-reranking|Context Reranking]] (CONCEPT)
- [[qwen3-reranker-series-0-6b-to-8b|Qwen3-Reranker Series (0.6B to 8B)]] (SYSTEM)
- [[qwen-2-5-0-5b|Qwen 2.5 (0.5B)]] (SYSTEM)
- [[retrieval-augmented-generation-rag-pipelines|Retrieval-Augmented Generation (RAG) pipelines]] (CONCEPT)
- [[gemini|Gemini]] (SYSTEM)
- [[output-verification|Output Verification]] (CONCEPT)

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
