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
- [[openclaw|OpenClaw]] (SYSTEM)
- [[heartbeat|Heartbeat]] (CONCEPT)
- [[cron|Cron]] (CONCEPT)
- [[node|Node]] (SYSTEM)
- [[claude-opus-4-6|Claude Opus 4.6]] (SYSTEM)
- [[docker|Docker]] (TOOL)
- [[hooks|Hooks]] (CONCEPT)
- [[skills|Skills]] (CONCEPT)
- [[sub-agent|Sub-agent]] (SYSTEM)
- [[gateway|Gateway]] (SYSTEM)
- [[aws-bedrock|AWS Bedrock]] (ORGANIZATION)
- [[plugins|Plugins]] (CONCEPT)
- [[telegram|Telegram]] (TOOL)
- [[loki|Loki]] (SYSTEM)

## Relations
- Loki → PART_OF → OpenClaw
- Loki → USES → Claude Opus 4.6
- Claude Opus 4.6 → RELATED_TO → AWS Bedrock
- OpenClaw → USES → Docker
- OpenClaw → USES → Telegram
