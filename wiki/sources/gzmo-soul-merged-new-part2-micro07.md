---
type: source
title: gzmo-soul-merged-new-part2-micro07
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# gzmo-soul-merged-new-part2-micro07

Ingested source summary (2026-06-10).

## Entities
- [[tale-ep-token-aware-llm-execution-via-prompting|TALE-EP (Token-Aware LLM Execution via Prompting)]] (SYSTEM)
- [[librarian-agent|Librarian Agent]] (SYSTEM)
- [[supervisoragent|SupervisorAgent]] (SYSTEM)
- [[fact-checker-agent|Fact-Checker Agent]] (SYSTEM)
- [[agentic-rag|Agentic RAG]] (SYSTEM)
- [[specialist-reader-agent|Specialist / Reader Agent]] (SYSTEM)
- [[gzmo|GZMO]] (PROJECT)
- [[editor-critic-agent|Editor / Critic Agent]] (SYSTEM)
- [[director-orchestrator-agent|Director / Orchestrator Agent]] (SYSTEM)
- [[model-context-protocol-mcp|Model Context Protocol (MCP)]] (SYSTEM)

## Relations
- Director / Orchestrator Agent → ORCHESTRATES → Librarian Agent
- Director / Orchestrator Agent → ORCHESTRATES → Specialist / Reader Agent
- Director / Orchestrator Agent → ORCHESTRATES → Fact-Checker Agent
- Director / Orchestrator Agent → ORCHESTRATES → Editor / Critic Agent
- SupervisorAgent → MONITORS_PURIFIES_DATA_FOR → Director / Orchestrator Agent
- Model Context Protocol (MCP) → PROVIDES_DYNAMIC_TOOL_SCHEMAS_TO → Director / Orchestrator Agent
