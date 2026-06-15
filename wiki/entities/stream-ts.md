---
type: entity
title: stream.ts
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# stream.ts

Type: SYSTEM

## From [[gzmo-daemon-validation-audit-and-bun-migration-rep|gzmo-daemon-validation-audit-and-bun-migration-rep]] (2026-06-08)
- Uses writeFileSync ×2, readFileSync ×1, existsSync ×1.
- Critical impact as the 60s LiveStream Pulse is blocked.
- Needs migration to Bun.file() / Bun.write().
