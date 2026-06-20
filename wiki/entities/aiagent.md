---
type: entity
title: AIAgent
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# AIAgent

Type: SYSTEM

## From [drive-research-hermes-agent-prompt-builder-analysis](/entities/drive-research-hermes-agent-prompt-builder-analysis.md) (2026-06-08)
- Core loop is in run_agent.py.
- Initializes with logic that can cause self.load_soul_identity to evaluate to false in gateway mode.
- Executes the run_conversation() loop to construct API payloads.
