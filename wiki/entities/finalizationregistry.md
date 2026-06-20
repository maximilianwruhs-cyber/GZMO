---
type: entity
title: FinalizationRegistry
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# FinalizationRegistry

Type: TOOL

## From [drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03](/entities/drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03.md) (2026-06-09)
- Relying on the FinalizationRegistry to trigger native cleanup callbacks is inherently dangerous, as V8 and JavaScriptCore garbage collection cycles are non-deterministic and provide no guarantees on execution timing.

## From [drive-research-bun-typescript-performance-tips-micro03](/entities/drive-research-bun-typescript-performance-tips-micro03.md) (2026-06-09)
- Relying on this to trigger native cleanup callbacks is dangerous.
