---
type: entity
title: Orchestrator-Agent
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Orchestrator-Agent

Type: SYSTEM

## From [aether-grid-micro01](/entities/aether-grid-micro01.md) (2026-06-09)
- Central agent that maintains the persona.
- Delegates tasks to specialized worker agents.
- Can get stuck in a feedback loop with worker agents.

## From [prompt-agent-engineering-part6-micro01](/entities/prompt-agent-engineering-part6-micro01.md) (2026-06-10)
- The entry point for queries.
- Breaks goals into a TaskList.
- Uses A2A to discover and call specialists.
