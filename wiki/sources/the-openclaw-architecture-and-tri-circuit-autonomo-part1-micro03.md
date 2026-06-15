---
type: source
title: the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro03

Ingested source summary (2026-06-09).

## Entities
- [[openclaw-rl|OpenClaw-RL]] (TOOL)
- [[verdict-answer-reasoning-schema|Verdict-Answer-Reasoning schema]] (CONCEPT)
- [[v3-prompt|v3 prompt]] (TOOL)
- [[kuadrant-authpolicies|Kuadrant AuthPolicies]] (TOOL)
- [[deepseek-v3-2-speciale|DeepSeek-V3.2-Speciale]] (SYSTEM)
- [[openclaw-agents-kit|openclaw-agents kit]] (TOOL)
- [[soul-md|SOUL.md]] (TOOL)
- [[critic|Critic]] (CONCEPT)
- [[mcp-gateways|MCP Gateways]] (SYSTEM)
- [[sharp-taste-gates|SHARP taste gates]] (CONCEPT)
- [[surveyor|Surveyor]] (CONCEPT)
- [[agents-md|AGENTS.md]] (TOOL)
- [[cron-scheduling|Cron scheduling]] (CONCEPT)
- [[heartbeats|Heartbeats]] (CONCEPT)
- [[mention-gating|Mention Gating]] (CONCEPT)
- [[ideator|Ideator]] (CONCEPT)
- [[evidence-hierarchy|Evidence Hierarchy]] (CONCEPT)
- [[docker-sandboxes|Docker sandboxes]] (SYSTEM)
- [[planner|Planner]] (CONCEPT)
- [[agenttoagent-tool|agentToAgent tool]] (TOOL)
- [[writer|Writer]] (CONCEPT)
- [[scout|Scout]] (CONCEPT)

## Relations
- Critic → USES → SHARP taste gates
- Critic → USES → Evidence Hierarchy
- Critic → RELATED_TO → Writer
- Planner → USES → agentToAgent tool
- Planner → RELATED_TO → Critic
- MCP Gateways → USES → Kuadrant AuthPolicies
