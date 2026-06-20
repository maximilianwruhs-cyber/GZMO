---
type: entity
title: confirm-destructive
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# confirm-destructive

Type: TOOL

## From [drive-research-pi-coding-agent-ecosystem-tier-list](/entities/drive-research-pi-coding-agent-ecosystem-tier-list.md) (2026-06-08)
- Implements a manual verification gate for potentially destructive shell tasks.
- Intercepts outgoing bash executions and evaluates the command string against a list of dangerous commands.
- Pauses execution and displays a manual confirmation prompt.

## From [drive-research-the-pi-coding-agent-s-architectural-paradigm-revol](/entities/drive-research-the-pi-coding-agent-s-architectural-paradigm-revol.md) (2026-06-08)
- Human-in-the-loop permission gates for dangerous actions.
- A-Tier resource.
- Implements tiered validation gates, forcing confirmation for dangerous commands.
