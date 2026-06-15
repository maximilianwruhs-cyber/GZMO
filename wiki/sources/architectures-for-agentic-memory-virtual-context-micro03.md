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
- [[response-format|response_format]] (TOOL)
- [[qwen-2-5-1-5b|Qwen 2.5 1.5B]] (SYSTEM)
- [[gguf|GGUF]] (CONCEPT)
- [[qwen-2-5-7b|Qwen 2.5 7B]] (SYSTEM)
- [[harmony-formatting|Harmony formatting]] (CONCEPT)
- [[extracted-data|extracted_data]] (CONCEPT)
- [[llama-3-1-8b|Llama 3.1 8B]] (SYSTEM)
- [[retrieval-augmented-generation|Retrieval-Augmented Generation]] (CONCEPT)
- [[gemma-3-4b|Gemma 3 4B]] (SYSTEM)
- [[ascentcore-small-llm-performance-benchmark|AscentCore Small LLM Performance Benchmark]] (TOOL)
- [[transformer-models|Transformer models]] (SYSTEM)
- [[gbnf|GBNF]] (TOOL)
- [[internal-analysis|internal_analysis]] (CONCEPT)
- [[crane-evaluation-framework|CRANE evaluation framework]] (TOOL)
- [[json-schema|JSON schema]] (CONCEPT)
- [[lm-studio|LM Studio]] (TOOL)
- [[mistral-7b|Mistral 7B]] (SYSTEM)
- [[smollm2-1-7b|SmolLM2 1.7B]] (SYSTEM)
- [[ijson|ijson]] (TOOL)

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
