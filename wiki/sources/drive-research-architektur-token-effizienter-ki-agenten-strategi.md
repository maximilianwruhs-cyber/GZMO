---
type: source
title: drive-research-architektur-token-effizienter-ki-agenten-strategi
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-architektur-token-effizienter-ki-agenten-strategi

Ingested source summary (2026-06-08).

## Entities
- [ReAct](/entities/react.md) (CONCEPT)
- [Hidden CoT](/entities/hidden-cot.md) (CONCEPT)
- [Prompt Caching](/entities/prompt-caching.md) (CONCEPT)
- [Token-Aware Chain-of-Thought Budgets (TALE)](/entities/token-aware-chain-of-thought-budgets-tale.md) (CONCEPT)
- [Retrieval-Augmented Generation (RAG)](/entities/retrieval-augmented-generation-rag.md) (CONCEPT)
- [Vector Memory](/entities/vector-memory.md) (SYSTEM)
- [Rolling Summarization](/entities/rolling-summarization.md) (CONCEPT)
- [SupervisorAgent](/entities/supervisoragent.md) (SYSTEM)
- [Tree of Thoughts (ToT)](/entities/tree-of-thoughts-tot.md) (CONCEPT)
- [Multi-Agent Systems (MAS)](/entities/multi-agent-systems-mas.md) (SYSTEM)
- [Supervised Multi-Agent System (SMAS)](/entities/supervised-multi-agent-system-smas.md) (SYSTEM)
- [CodeAgents](/entities/codeagents.md) (SYSTEM)
- [Gist-Tokens](/entities/gist-tokens.md) (CONCEPT)
- [Selective CoT](/entities/selective-cot.md) (CONCEPT)
- [Key-Value (KV) Store](/entities/key-value-kv-store.md) (SYSTEM)
- [Function Calling](/entities/function-calling.md) (CONCEPT)
- [Model Context Protocol (MCP)](/entities/model-context-protocol-mcp.md) (CONCEPT)
- [Claude Haiku](/entities/claude-haiku.md) (SYSTEM)
- [SQL-Based Memory](/entities/sql-based-memory.md) (SYSTEM)
- [LLMLingua-2](/entities/llmlingua-2.md) (TOOL)

## Relations
- ReAct → PART_OF → Multi-Agent Systems (MAS)
- Function Calling → RELATED_TO → ReAct
- CodeAgents → PART_OF → Multi-Agent Systems (MAS)
- Token-Aware Chain-of-Thought Budgets (TALE) → RELATED_TO → Multi-Agent Systems (MAS)
- Selective CoT → RELATED_TO → Token-Aware Chain-of-Thought Budgets (TALE)
- Hidden CoT → RELATED_TO → Token-Aware Chain-of-Thought Budgets (TALE)
- Tree of Thoughts (ToT) → RELATED_TO → Token-Aware Chain-of-Thought Budgets (TALE)
- Retrieval-Augmented Generation (RAG) → RELATED_TO → Multi-Agent Systems (MAS)
- Rolling Summarization → USES → Claude Haiku
- Supervised Multi-Agent System (SMAS) → RELATED_TO → Multi-Agent Systems (MAS)
- SupervisorAgent → PART_OF → Supervised Multi-Agent System (SMAS)
- Model Context Protocol (MCP) → RELATED_TO → Function Calling
