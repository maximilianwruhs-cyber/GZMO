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
- [OpenClaw-RL](/entities/openclaw-rl.md) (TOOL)
- [Verdict-Answer-Reasoning schema](/entities/verdict-answer-reasoning-schema.md) (CONCEPT)
- [v3 prompt](/entities/v3-prompt.md) (TOOL)
- [Kuadrant AuthPolicies](/entities/kuadrant-authpolicies.md) (TOOL)
- [DeepSeek-V3.2-Speciale](/entities/deepseek-v3-2-speciale.md) (SYSTEM)
- [openclaw-agents kit](/entities/openclaw-agents-kit.md) (TOOL)
- [SOUL.md](/entities/soul-md.md) (TOOL)
- [Critic](/entities/critic.md) (CONCEPT)
- [MCP Gateways](/entities/mcp-gateways.md) (SYSTEM)
- [SHARP taste gates](/entities/sharp-taste-gates.md) (CONCEPT)
- [Surveyor](/entities/surveyor.md) (CONCEPT)
- [AGENTS.md](/entities/agents-md.md) (TOOL)
- [Cron scheduling](/entities/cron-scheduling.md) (CONCEPT)
- [Heartbeats](/entities/heartbeats.md) (CONCEPT)
- [Mention Gating](/entities/mention-gating.md) (CONCEPT)
- [Ideator](/entities/ideator.md) (CONCEPT)
- [Evidence Hierarchy](/entities/evidence-hierarchy.md) (CONCEPT)
- [Docker sandboxes](/entities/docker-sandboxes.md) (SYSTEM)
- [Planner](/entities/planner.md) (CONCEPT)
- [agentToAgent tool](/entities/agenttoagent-tool.md) (TOOL)
- [Writer](/entities/writer.md) (CONCEPT)
- [Scout](/entities/scout.md) (CONCEPT)

## Relations
- Critic → USES → SHARP taste gates
- Critic → USES → Evidence Hierarchy
- Critic → RELATED_TO → Writer
- Planner → USES → agentToAgent tool
- Planner → RELATED_TO → Critic
- MCP Gateways → USES → Kuadrant AuthPolicies
