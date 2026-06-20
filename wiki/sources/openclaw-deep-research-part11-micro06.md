---
type: source
title: openclaw-deep-research-part11-micro06
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# openclaw-deep-research-part11-micro06

Ingested source summary (2026-06-09).

## Entities
- [Scheduled Tasks (Cron)](/entities/scheduled-tasks-cron.md) (TOOL)
- [GlobalSettings](/entities/globalsettings.md) (CONCEPT)
- [Hooks](/entities/hooks.md) (CONCEPT)
- [Background Tasks](/entities/background-tasks.md) (CONCEPT)
- [GatewayConfig](/entities/gatewayconfig.md) (CONCEPT)
- [openclaw_core](/entities/openclaw-core.md) (PROJECT)
- [JSON5](/entities/json5.md) (CONCEPT)
- [Task Flow](/entities/task-flow.md) (CONCEPT)
- [Heartbeat](/entities/heartbeat.md) (CONCEPT)
- [AGENTS.md](/entities/agents-md.md) (BOOK)
- [Rust](/entities/rust.md) (CONCEPT)
- [Config](/entities/config.md) (CONCEPT)
- [ChannelsConfig](/entities/channelsconfig.md) (CONCEPT)
- [ProvidersConfig](/entities/providersconfig.md) (CONCEPT)
- [Standing Orders](/entities/standing-orders.md) (CONCEPT)
- [AgentConfig](/entities/agentconfig.md) (CONCEPT)

## Relations
- openclaw_core → USES → Scheduled Tasks (Cron)
- openclaw_core → USES → Heartbeat
- openclaw_core → USES → Background Tasks
- openclaw_core → USES → Task Flow
- openclaw_core → USES → Standing Orders
- openclaw_core → USES → Hooks
- openclaw_core → USES → Config
- openclaw_core → USES → JSON5
- Config → PART_OF → GatewayConfig
- Config → PART_OF → AgentConfig
- Config → PART_OF → ChannelsConfig
- Config → PART_OF → ProvidersConfig
- Config → PART_OF → GlobalSettings
- openclaw_core → RELATED_TO → Rust
- Config → RELATED_TO → JSON5
- Standing Orders → PART_OF → AGENTS.md
