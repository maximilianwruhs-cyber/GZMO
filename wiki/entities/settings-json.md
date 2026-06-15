---
type: entity
title: settings.json
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# settings.json

Type: TOOL

## From [[drive-research-inside-the-pi-coding-agent-optimization-isn|drive-research-inside-the---pi---coding-agent--optimization--isn]] (2026-06-08)
- Used to explicitly define tools available to the agent.
- Stripping unnecessary tools reduces the LLM's decision surface.

## From [[drive-research-optimizing-pi-coding-agent-performance|drive-research-optimizing-pi-coding-agent-performance]] (2026-06-08)
- Used to restrict and define available tools.
- Can be global (~/.pi/agent/settings.json) or project-local (.pi/settings.json).
- Ensures the model executes only necessary operations.
