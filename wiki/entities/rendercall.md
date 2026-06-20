---
type: entity
title: renderCall
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# renderCall

Type: TOOL

## From [drive-research-agentic-typescript-monorepo-context-management](/entities/drive-research-agentic-typescript-monorepo-context-management.md) (2026-06-08)
- Function allowing developers to intercept raw JSON input/output of a tool.
- Converts input/output into visually coherent markdown, tables, or syntax blocks.

## From [drive-research-building-pi-coding-agent-extensions](/entities/drive-research-building-pi-coding-agent-extensions.md) (2026-06-08)
- Responsible for painting the tool's header, parsing streaming arguments, and displaying visual indicators while the execute block is running.
- Strictly required to return an object that implements the TUI Component interface.
