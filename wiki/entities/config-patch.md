---
type: entity
title: config.patch
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# config.patch

Type: TOOL

## From [openclaw-deep-research-part1-micro03](/entities/openclaw-deep-research-part1-micro03.md) (2026-06-09)
- Control-plane write RPC for programmatic configuration updates.
- Preferred for partial updates.
- Used after config.schema.lookup.

## From [openclaw-deep-research-part1-micro04](/entities/openclaw-deep-research-part1-micro04.md) (2026-06-09)
- Called by openclaw gateway.
- Performs partial updates to configuration.
- Uses JSON merge patch semantics.
- Requires raw and baseHash parameters.

## From [openclaw-deep-research-part9-micro06](/entities/openclaw-deep-research-part9-micro06.md) (2026-06-09)
- Control-plane write RPC
- Preferred partial update path
- Merges a partial update into the existing config
