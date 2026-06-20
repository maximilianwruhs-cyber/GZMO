---
type: entity
title: skills.ts
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# skills.ts

Type: SYSTEM

## From [gzmo-daemon-validation-audit-and-bun-migration-rep](/entities/gzmo-daemon-validation-audit-and-bun-migration-rep.md) (2026-06-08)
- Uses readdirSync, readFileSync, existsSync.
- Low impact, only affects boot-time.
- Needs migration to Bun.file() / Bun.write().
