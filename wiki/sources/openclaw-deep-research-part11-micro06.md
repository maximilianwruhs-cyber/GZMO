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
- [[scheduled-tasks-cron|Scheduled Tasks (Cron)]] (TOOL)
- [[globalsettings|GlobalSettings]] (CONCEPT)
- [[hooks|Hooks]] (CONCEPT)
- [[background-tasks|Background Tasks]] (CONCEPT)
- [[gatewayconfig|GatewayConfig]] (CONCEPT)
- [[openclaw-core|openclaw_core]] (PROJECT)
- [[json5|JSON5]] (CONCEPT)
- [[task-flow|Task Flow]] (CONCEPT)
- [[heartbeat|Heartbeat]] (CONCEPT)
- [[agents-md|AGENTS.md]] (BOOK)
- [[rust|Rust]] (CONCEPT)
- [[config|Config]] (CONCEPT)
- [[channelsconfig|ChannelsConfig]] (CONCEPT)
- [[providersconfig|ProvidersConfig]] (CONCEPT)
- [[standing-orders|Standing Orders]] (CONCEPT)
- [[agentconfig|AgentConfig]] (CONCEPT)

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
