---
type: entity
title: config.apply
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# config.apply

Type: TOOL

## From [openclaw-deep-research-part1-micro03](/entities/openclaw-deep-research-part1-micro03.md) (2026-06-09)
- Control-plane write RPC for programmatic configuration updates.
- Performs full-config replacement and restarts the Gateway.
- Validates and writes the full config in one step.

## From [openclaw-deep-research-part1-micro04](/entities/openclaw-deep-research-part1-micro04.md) (2026-06-09)
- Called by openclaw gateway.
- Applies configuration with raw, baseHash, and sessionKey.
- Has specific restart behavior.

## From [openclaw-deep-research-part9-micro06](/entities/openclaw-deep-research-part9-micro06.md) (2026-06-09)
- Control-plane write RPC
- Performs full-config replacement and restarts the Gateway
