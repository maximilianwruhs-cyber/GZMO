---
type: entity
title: TelemetryService
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# TelemetryService

Type: SYSTEM

## From [obolus-vs-codium-extension-konzept-research-part1-micro05](/entities/obolus-vs-codium-extension-konzept-research-part1-micro05.md) (2026-06-09)
- Dedicated service in `src/services/TelemetryService.ts`.
- Holds WebSocket connection, handles disconnects, buffers incoming data.
- Implements throttling and broadcasting to webview.
