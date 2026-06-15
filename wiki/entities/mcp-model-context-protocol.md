---
type: entity
title: MCP (Model Context Protocol)
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# MCP (Model Context Protocol)

Type: CONCEPT

## From [[openclaw-deep-research-part11-micro04|openclaw-deep-research-part11-micro04]] (2026-06-09)
- A standardized tool layer that connects the agent to external services.
- Exposes a set of tools with defined schemas.
- The agent discovers available tools, calls them using a standard request format, and receives a structured result.
- Provides tool portability: tools built for one MCP-compatible agent can be reused across other systems.

## From [[prompt-agent-engineering-part6-micro01|prompt-agent-engineering-part6-micro01]] (2026-06-10)
- Acts as a capability interface for vertical tool integration.
- Provides standardized tool and resource access.
- Enables dynamic tool listing and invocation.
