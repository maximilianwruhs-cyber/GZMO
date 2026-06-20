---
type: source
title: architectures-for-agentic-memory-virtual-context-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# architectures-for-agentic-memory-virtual-context-micro03

Ingested source summary (2026-06-09).

## Entities
- [response_format](/entities/response-format.md) (TOOL)
- [Qwen 2.5 1.5B](/entities/qwen-2-5-1-5b.md) (SYSTEM)
- [GGUF](/entities/gguf.md) (CONCEPT)
- [Qwen 2.5 7B](/entities/qwen-2-5-7b.md) (SYSTEM)
- [Harmony formatting](/entities/harmony-formatting.md) (CONCEPT)
- [extracted_data](/entities/extracted-data.md) (CONCEPT)
- [Llama 3.1 8B](/entities/llama-3-1-8b.md) (SYSTEM)
- [Retrieval-Augmented Generation](/entities/retrieval-augmented-generation.md) (CONCEPT)
- [Gemma 3 4B](/entities/gemma-3-4b.md) (SYSTEM)
- [AscentCore Small LLM Performance Benchmark](/entities/ascentcore-small-llm-performance-benchmark.md) (TOOL)
- [Transformer models](/entities/transformer-models.md) (SYSTEM)
- [GBNF](/entities/gbnf.md) (TOOL)
- [internal_analysis](/entities/internal-analysis.md) (CONCEPT)
- [CRANE evaluation framework](/entities/crane-evaluation-framework.md) (TOOL)
- [JSON schema](/entities/json-schema.md) (CONCEPT)
- [LM Studio](/entities/lm-studio.md) (TOOL)
- [Mistral 7B](/entities/mistral-7b.md) (SYSTEM)
- [SmolLM2 1.7B](/entities/smollm2-1-7b.md) (SYSTEM)
- [ijson](/entities/ijson.md) (TOOL)

## Relations
- GBNF → USES → JSON schema
- response_format → USES → JSON schema
- CRANE evaluation framework → RELATED_TO → Transformer models
- JSON schema → PART_OF → internal_analysis
- JSON schema → PART_OF → extracted_data
- AscentCore Small LLM Performance Benchmark → RELATED_TO → Llama 3.1 8B
- AscentCore Small LLM Performance Benchmark → RELATED_TO → Qwen 2.5 7B
- AscentCore Small LLM Performance Benchmark → RELATED_TO → Mistral 7B
- AscentCore Small LLM Performance Benchmark → RELATED_TO → SmolLM2 1.7B
- AscentCore Small LLM Performance Benchmark → RELATED_TO → Qwen 2.5 1.5B
- AscentCore Small LLM Performance Benchmark → RELATED_TO → Gemma 3 4B
- Llama 3.1 8B → USES → JSON schema
- Qwen 2.5 7B → USES → JSON schema
- Mistral 7B → USES → JSON schema
- SmolLM2 1.7B → USES → JSON schema
- Qwen 2.5 1.5B → USES → JSON schema
- Gemma 3 4B → USES → JSON schema
- LM Studio → USES → GBNF
- LM Studio → RELATED_TO → Harmony formatting
- ijson → USES → JSON
- GGUF → USES → JSON
