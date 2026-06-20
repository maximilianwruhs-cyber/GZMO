---
type: entity
title: gzmo_watchdog.py
created: 2026-06-09
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# gzmo_watchdog.py

Type: TOOL

## From [prompt-agent-engineering-part4-micro04](/entities/prompt-agent-engineering-part4-micro04.md) (2026-06-09)
- Heartbeat script

## From [prompt-agent-engineering-part4-micro02](/entities/prompt-agent-engineering-part4-micro02.md) (2026-06-10)
- Mentioned as a specific piece of deployment logic to drill into

## From [prompt-agent-engineering-part4-micro06](/entities/prompt-agent-engineering-part4-micro06.md) (2026-06-10)
- Core Orchestrator
- Generates DAG JSON payloads
- Reads TELEMETRY.json to minimize token usage
- Controls the system lifecycle
- Runs telemetry compilation, GZMO inference, and DAG execution
- Operates in a loop with a 4-hour interval
