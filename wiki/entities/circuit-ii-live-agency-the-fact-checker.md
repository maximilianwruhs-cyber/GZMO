---
type: entity
title: 'Circuit II: Live Agency (The Fact-Checker)'
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Circuit II: Live Agency (The Fact-Checker)

Type: SYSTEM

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part2](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part2.md) (2026-06-08)
- Purpose: The production-facing operational agent that processes queries and delivers judgments.
- Stack: OpenClaw-RL deployed as a systemd service, bridging to local LLMs via LM Studio.
- Governed by the FACT-CHECKER AGENT PROMPT (v3).
