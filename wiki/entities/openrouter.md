---
type: entity
title: OpenRouter
created: 2026-06-08
updated: 2026-06-10
sources: 10
tags: []
status: draft
gzmo_synthetic: true
---










# OpenRouter

Type: SYSTEM

## From [openclaw-part2](/entities/openclaw-part2.md) (2026-06-08)
- A complex routing portal that Hermes Agent can use to hot-swap models mid-workflow.

## From [openclaw-deep-research-part12](/entities/openclaw-deep-research-part12.md) (2026-06-08)
- A provider where costs can be monitored.
- Has an activity dashboard.

## From [drive-research-hermes-compression-and-bol-architecture](/entities/drive-research-hermes-compression-and-bol-architecture.md) (2026-06-08)
- Provider through which specialized summarization models can be mapped.

## From [drive-research-hermes-session-storage-migration-analysis](/entities/drive-research-hermes-session-storage-migration-analysis.md) (2026-06-08)
- Provides access to models with different pricing structures.
- Used for the three-tier model cascade in Hermes.

## From [drive-research-hermes-anthropic-openrouter-cache-investigation](/entities/drive-research-hermes-anthropic-openrouter-cache-investigation.md) (2026-06-08)
- It is an aggregator service that theoretically supports prompt caching for Anthropic models.
- It uses a mechanism called 'Provider Sticky Routing' for caching.
- It exhibits a serialization error in the chat_completions mode that causes caching markers to be ignored.
- Has anomalies in routing.
- Suffers from serialization errors in chat_completions mode.
- Can lead to unacknowledged tenfold increases in token costs.

## From [drive-research-pi-coding-agent-local-deployment-customization](/entities/drive-research-pi-coding-agent-local-deployment-customization.md) (2026-06-08)
- Provider name for the openai-completions API wire protocol.

## From [openclaw-deep-research-part7-micro02](/entities/openclaw-deep-research-part7-micro02.md) (2026-06-10)
- Required for agent-swarm

## From [openclaw-deep-research-part8-micro06](/entities/openclaw-deep-research-part8-micro06.md) (2026-06-10)
- A model provider used in configurations

## From [openclaw-deep-research-part9-micro03](/entities/openclaw-deep-research-part9-micro03.md) (2026-06-10)
- Multi-provider router providing access to 100+ models
- Uses API keys starting with sk-or-

## From [openclaw-deep-research-part9-micro04](/entities/openclaw-deep-research-part9-micro04.md) (2026-06-10)
- A service that allows routing across multiple providers using a single key
