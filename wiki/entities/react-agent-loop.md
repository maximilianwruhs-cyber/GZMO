---
type: entity
title: ReAct Agent Loop
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# ReAct Agent Loop

Type: CONCEPT

## From [prompt-agent-engineering-part5-micro04](/entities/prompt-agent-engineering-part5-micro04.md) (2026-06-09)
- Described as the 'brain' of the system.
- Involves searching memory, LLM decision, tool calls, and reflection.
- Implemented with a maximum iteration limit.

## From [prompt-agent-engineering-part5-micro05](/entities/prompt-agent-engineering-part5-micro05.md) (2026-06-09)
- Uses Message-Array
- Uses Ollama Tool-Calling / Structured Outputs
- String concatenation is fragile
- Ollama supports tool-calling since ~0.3+ (2025)
