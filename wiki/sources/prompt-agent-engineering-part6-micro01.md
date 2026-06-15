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
- [[capability-layer|Capability Layer]] (CONCEPT)
- [[python|Python]] (TOOL)
- [[mcp-model-context-protocol|MCP (Model Context Protocol)]] (CONCEPT)
- [[orchestration-layer|Orchestration Layer]] (CONCEPT)
- [[agent-cards|Agent Cards]] (CONCEPT)
- [[arize-phoenix|Arize Phoenix]] (TOOL)
- [[fastmcp|FastMCP]] (TOOL)
- [[mcp-server|MCP Server]] (SYSTEM)
- [[docker|Docker]] (TOOL)
- [[orchestrator-agent|Orchestrator Agent]] (SYSTEM)
- [[opentelemetry|OpenTelemetry]] (TOOL)
- [[redis|Redis]] (TOOL)
- [[communication-layer|Communication Layer]] (CONCEPT)
- [[google-adk|Google ADK]] (TOOL)
- [[a2a-agent-to-agent-protocol|A2A (Agent-to-Agent) protocol]] (CONCEPT)
- [[postgresql|PostgreSQL]] (TOOL)
- [[gemini|Gemini]] (SYSTEM)
- [[kubernetes|Kubernetes]] (TOOL)
- [[langsmith|LangSmith]] (TOOL)
- [[specialist-agents|Specialist Agents]] (SYSTEM)

## Relations
- Orchestration Layer → USES → A2A (Agent-to-Agent) protocol
- Communication Layer → USES → A2A (Agent-to-Agent) protocol
- Capability Layer → USES → MCP (Model Context Protocol)
- Orchestrator Agent → USES → A2A (Agent-to-Agent) protocol
- Specialist Agents → USES → MCP (Model Context Protocol)
- Specialist Agents → USES → A2A (Agent-to-Agent) protocol
- MCP Server → USES → FastMCP
- Specialist Agents → USES → Google ADK
