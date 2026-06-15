---
type: entity
title: engine.ts
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# engine.ts

Type: SYSTEM

## From [[gzmo-daemon-validation-audit-and-bun-migration-rep|gzmo-daemon-validation-audit-and-bun-migration-rep]] (2026-06-08)
- Uses writeFileSync (chain action) at line 228.
- Medium impact, only affects 'action: chain'.
- Needs migration to Bun.file() / Bun.write().

## From [[gzmo-chaos-engine-architecture-audit-and-behaviora|gzmo-chaos-engine-architecture-audit-and-behaviora]] (2026-06-08)
- Processes tasks using processTask()
- Currently only uses llmTemperature from the attractor's outputs
- Modifies system prompt based on phase
