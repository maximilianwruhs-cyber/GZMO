---
type: entity
title: frontmatter.ts
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# frontmatter.ts

Type: SYSTEM

## From [gzmo-daemon-validation-audit-and-bun-migration-rep](/entities/gzmo-daemon-validation-audit-and-bun-migration-rep.md) (2026-06-08)
- Uses readFileSync ×3, writeFileSync ×2.
- Critical impact as it's called for EVERY task, blocking the Event Loop.
- Needs migration to Bun.file() / Bun.write().
