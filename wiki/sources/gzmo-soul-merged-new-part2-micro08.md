---
type: source
title: gzmo-soul-merged-new-part2-micro08
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# gzmo-soul-merged-new-part2-micro08

Ingested source summary (2026-06-10).

## Entities
- [LLM-as-a-Judge](/entities/llm-as-a-judge.md) (ARCHITECTURE_PATTERN)
- [GZMO](/entities/gzmo.md) (AGENT)
- [Firecracker](/entities/firecracker.md) (TECHNOLOGY)
- [CodeAct](/entities/codeact.md) (PARADIGM)
- [Supervised Multi-Agent System (SMAS)](/entities/supervised-multi-agent-system-smas.md) (ARCHITECTURE_PATTERN)
- [Fact-Checker](/entities/fact-checker.md) (AGENT_ROLE)
- [spec.md](/entities/spec-md.md) (DOCUMENT)
- [SOUL.md](/entities/soul-md.md) (DOCUMENT)
- [memory/](/entities/memory.md) (DIRECTORY)
- [Reader Agent](/entities/reader-agent.md) (AGENT_ROLE)
- [Editor](/entities/editor.md) (AGENT_ROLE)
- [Multi-Agent Topology](/entities/multi-agent-topology.md) (ARCHITECTURE_PATTERN)
- [SKILL.md](/entities/skill-md.md) (DOCUMENT)
- [Model Context Protocol (MCP)](/entities/model-context-protocol-mcp.md) (PROTOCOL)
- [Librarian Agent](/entities/librarian-agent.md) (AGENT_ROLE)
- [SupervisorAgent](/entities/supervisoragent.md) (AGENT_ROLE)

## Relations
- GZMO → IS_DEFINED_BY → SOUL.md
- GZMO → DELEGATES_TOOLS_TO → SKILL.md
- GZMO → ROUTES_TASKS_TO → Librarian Agent
- GZMO → ROUTES_TASKS_TO → Reader Agent
- GZMO → ROUTES_TASKS_TO → Fact-Checker
- GZMO → ROUTES_TASKS_TO → Editor
- GZMO → USES_FOR_PURIFICATION → SupervisorAgent
