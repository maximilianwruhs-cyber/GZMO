---
type: entity
title: heartbeat
created: 2026-06-09
updated: 2026-06-10
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# heartbeat

Type: CONCEPT

## From [architectures-for-agentic-memory-virtual-context-micro06](/entities/architectures-for-agentic-memory-virtual-context-micro06.md) (2026-06-09)
- Internal system message injected after a tool call.
- Signaled the agent to execute another inference cycle autonomously.
- Deprecated in Letta V1.

## From [openclaw-deep-research-part11-micro06](/entities/openclaw-deep-research-part11-micro06.md) (2026-06-09)
- A periodic main-session turn (default every 30 minutes).
- Batches multiple checks (inbox, calendar, notifications) in one agent turn with full session context.
- Heartbeat turns do not create task records.
- Used when work benefits from full session context and approximate timing is fine.

## From [openclaw-deep-research-part9-micro02](/entities/openclaw-deep-research-part9-micro02.md) (2026-06-10)
- Periodic agent turns in the main session
- Can trigger background work like memory maintenance

## From [openclaw-part1-micro01](/entities/openclaw-part1-micro01.md) (2026-06-10)
- Acts as a deterministic gatekeeper
- Uses lightweight scripts to decide if information requires heavy AI models
- Functions as the digital equivalent of the sensory register
