---
type: source
title: openclaw-deep-research-part9-micro02
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# openclaw-deep-research-part9-micro02

Ingested source summary (2026-06-10).

## Entities
- [OpenClaw](/entities/openclaw.md) (SYSTEM)
- [Heartbeat](/entities/heartbeat.md) (CONCEPT)
- [Cron](/entities/cron.md) (CONCEPT)
- [Node](/entities/node.md) (SYSTEM)
- [Claude Opus 4.6](/entities/claude-opus-4-6.md) (SYSTEM)
- [Docker](/entities/docker.md) (TOOL)
- [Hooks](/entities/hooks.md) (CONCEPT)
- [Skills](/entities/skills.md) (CONCEPT)
- [Sub-agent](/entities/sub-agent.md) (SYSTEM)
- [Gateway](/entities/gateway.md) (SYSTEM)
- [AWS Bedrock](/entities/aws-bedrock.md) (ORGANIZATION)
- [Plugins](/entities/plugins.md) (CONCEPT)
- [Telegram](/entities/telegram.md) (TOOL)
- [Loki](/entities/loki.md) (SYSTEM)

## Relations
- Loki → PART_OF → OpenClaw
- Loki → USES → Claude Opus 4.6
- Claude Opus 4.6 → RELATED_TO → AWS Bedrock
- OpenClaw → USES → Docker
- OpenClaw → USES → Telegram
