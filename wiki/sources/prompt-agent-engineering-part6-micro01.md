---
type: source
title: prompt-agent-engineering-part6-micro01
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# prompt-agent-engineering-part6-micro01

Ingested source summary (2026-06-10).

## Entities
- [Capability Layer](/entities/capability-layer.md) (CONCEPT)
- [Python](/entities/python.md) (TOOL)
- [MCP (Model Context Protocol)](/entities/mcp-model-context-protocol.md) (CONCEPT)
- [Orchestration Layer](/entities/orchestration-layer.md) (CONCEPT)
- [Agent Cards](/entities/agent-cards.md) (CONCEPT)
- [Arize Phoenix](/entities/arize-phoenix.md) (TOOL)
- [FastMCP](/entities/fastmcp.md) (TOOL)
- [MCP Server](/entities/mcp-server.md) (SYSTEM)
- [Docker](/entities/docker.md) (TOOL)
- [Orchestrator Agent](/entities/orchestrator-agent.md) (SYSTEM)
- [OpenTelemetry](/entities/opentelemetry.md) (TOOL)
- [Redis](/entities/redis.md) (TOOL)
- [Communication Layer](/entities/communication-layer.md) (CONCEPT)
- [Google ADK](/entities/google-adk.md) (TOOL)
- [A2A (Agent-to-Agent) protocol](/entities/a2a-agent-to-agent-protocol.md) (CONCEPT)
- [PostgreSQL](/entities/postgresql.md) (TOOL)
- [Gemini](/entities/gemini.md) (SYSTEM)
- [Kubernetes](/entities/kubernetes.md) (TOOL)
- [LangSmith](/entities/langsmith.md) (TOOL)
- [Specialist Agents](/entities/specialist-agents.md) (SYSTEM)

## Relations
- Orchestration Layer → USES → A2A (Agent-to-Agent) protocol
- Communication Layer → USES → A2A (Agent-to-Agent) protocol
- Capability Layer → USES → MCP (Model Context Protocol)
- Orchestrator Agent → USES → A2A (Agent-to-Agent) protocol
- Specialist Agents → USES → MCP (Model Context Protocol)
- Specialist Agents → USES → A2A (Agent-to-Agent) protocol
- MCP Server → USES → FastMCP
- Specialist Agents → USES → Google ADK
