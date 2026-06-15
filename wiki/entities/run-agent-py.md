---
type: entity
title: run_agent.py
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# run_agent.py

Type: TOOL

## From [[drive-research-hermes-agent-prompt-builder-analysis|drive-research-hermes-agent-prompt-builder-analysis]] (2026-06-08)
- Contains the AIAgent core loop.
- Houses the _build_system_prompt() method which orchestrates prompt assembly.
- The prompt_parts array is finalized here.

## From [[drive-research-hermes-anthropic-openrouter-cache-investigation|drive-research-hermes-anthropic-openrouter-cache-investigation]] (2026-06-08)
- The TTL value for caching is hardcoded to '5m' in this file.
- The system defaults to the chat_completions API mode when the base URL does not end in /anthropic.
