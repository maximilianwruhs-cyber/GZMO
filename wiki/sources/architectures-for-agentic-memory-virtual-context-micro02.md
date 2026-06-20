---
type: source
title: architectures-for-agentic-memory-virtual-context-micro02
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# architectures-for-agentic-memory-virtual-context-micro02

Ingested source summary (2026-06-09).

## Entities
- [json_schema_to_grammar.py](/entities/json-schema-to-grammar-py.md) (TOOL)
- [LM Studio](/entities/lm-studio.md) (SYSTEM)
- [OpenAI](/entities/openai.md) (ORGANIZATION)
- [LangChain](/entities/langchain.md) (TOOL)
- [LLGuidance](/entities/llguidance.md) (TOOL)
- [msgspec](/entities/msgspec.md) (TOOL)
- [XGrammar](/entities/xgrammar.md) (TOOL)
- [MLX](/entities/mlx.md) (SYSTEM)
- [Outlines](/entities/outlines.md) (TOOL)
- [pydantic](/entities/pydantic.md) (TOOL)
- [GGUF](/entities/gguf.md) (CONCEPT)
- [GGML Backus-Naur Form (GBNF)](/entities/ggml-backus-naur-form-gbnf.md) (CONCEPT)
- [llama.cpp](/entities/llama-cpp.md) (SYSTEM)

## Relations
- LM Studio → USES → llama.cpp
- LM Studio → USES → MLX
- GGML Backus-Naur Form (GBNF) → PART_OF → llama.cpp
- json_schema_to_grammar.py → RELATED_TO → GGML Backus-Naur Form (GBNF)
- LM Studio → RELATED_TO → OpenAI
- pydantic → USES → LM Studio
- msgspec → USES → LM Studio
