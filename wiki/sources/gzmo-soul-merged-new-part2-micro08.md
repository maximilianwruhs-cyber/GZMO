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
- [[llm-as-a-judge|LLM-as-a-Judge]] (ARCHITECTURE_PATTERN)
- [[gzmo|GZMO]] (AGENT)
- [[firecracker|Firecracker]] (TECHNOLOGY)
- [[codeact|CodeAct]] (PARADIGM)
- [[supervised-multi-agent-system-smas|Supervised Multi-Agent System (SMAS)]] (ARCHITECTURE_PATTERN)
- [[fact-checker|Fact-Checker]] (AGENT_ROLE)
- [[spec-md|spec.md]] (DOCUMENT)
- [[soul-md|SOUL.md]] (DOCUMENT)
- [[memory|memory/]] (DIRECTORY)
- [[reader-agent|Reader Agent]] (AGENT_ROLE)
- [[editor|Editor]] (AGENT_ROLE)
- [[multi-agent-topology|Multi-Agent Topology]] (ARCHITECTURE_PATTERN)
- [[skill-md|SKILL.md]] (DOCUMENT)
- [[model-context-protocol-mcp|Model Context Protocol (MCP)]] (PROTOCOL)
- [[librarian-agent|Librarian Agent]] (AGENT_ROLE)
- [[supervisoragent|SupervisorAgent]] (AGENT_ROLE)

## Relations
- GZMO → IS_DEFINED_BY → SOUL.md
- GZMO → DELEGATES_TOOLS_TO → SKILL.md
- GZMO → ROUTES_TASKS_TO → Librarian Agent
- GZMO → ROUTES_TASKS_TO → Reader Agent
- GZMO → ROUTES_TASKS_TO → Fact-Checker
- GZMO → ROUTES_TASKS_TO → Editor
- GZMO → USES_FOR_PURIFICATION → SupervisorAgent
