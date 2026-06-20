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
- [TALE-EP (Token-Aware LLM Execution via Prompting)](/entities/tale-ep-token-aware-llm-execution-via-prompting.md) (SYSTEM)
- [Librarian Agent](/entities/librarian-agent.md) (SYSTEM)
- [SupervisorAgent](/entities/supervisoragent.md) (SYSTEM)
- [Fact-Checker Agent](/entities/fact-checker-agent.md) (SYSTEM)
- [Agentic RAG](/entities/agentic-rag.md) (SYSTEM)
- [Specialist / Reader Agent](/entities/specialist-reader-agent.md) (SYSTEM)
- [GZMO](/entities/gzmo.md) (PROJECT)
- [Editor / Critic Agent](/entities/editor-critic-agent.md) (SYSTEM)
- [Director / Orchestrator Agent](/entities/director-orchestrator-agent.md) (SYSTEM)
- [Model Context Protocol (MCP)](/entities/model-context-protocol-mcp.md) (SYSTEM)

## Relations
- Director / Orchestrator Agent → ORCHESTRATES → Librarian Agent
- Director / Orchestrator Agent → ORCHESTRATES → Specialist / Reader Agent
- Director / Orchestrator Agent → ORCHESTRATES → Fact-Checker Agent
- Director / Orchestrator Agent → ORCHESTRATES → Editor / Critic Agent
- SupervisorAgent → MONITORS_PURIFIES_DATA_FOR → Director / Orchestrator Agent
- Model Context Protocol (MCP) → PROVIDES_DYNAMIC_TOOL_SCHEMAS_TO → Director / Orchestrator Agent
