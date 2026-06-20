---
type: entity
title: prompt_builder.py
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# prompt_builder.py

Type: TOOL

## From [drive-research-hermes-agent-prompt-builder-analysis](/entities/drive-research-hermes-agent-prompt-builder-analysis.md) (2026-06-08)
- Operates in conjunction with the AIAgent core loop in run_agent.py.
- Separates cached system prompt state from dynamic additions required at API-call time.
- Responsible for the sequential, layer-by-layer aggregation of distinct Python string constants, dynamically loaded file contents, and runtime context.
- Appends directives to the agent.
- Injects Platform Hints at Layer 10.
- Architecture must evolve toward strict XML encapsulation.
