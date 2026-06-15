---
type: entity
title: Context Scanner
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Context Scanner

Type: TOOL

## From [[drive-research-hermes-agent-prompt-builder-analysis|drive-research-hermes-agent-prompt-builder-analysis]] (2026-06-08)
- Primary defensive perimeter within prompt_builder.py.
- Acts as an internal Web Application Firewall (WAF) for the system prompt.
- Operates on a static list of regular expressions defined in _CONTEXT_THREAT_PATTERNS.
- Actively blocks overt phrases such as 'ignore previous instructions', 'do not tell the user', 'system prompt override'.
