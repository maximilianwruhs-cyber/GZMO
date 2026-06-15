---
type: entity
title: embeddings.ts
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# embeddings.ts

Type: SYSTEM

## From [[gzmo-daemon-validation-audit-and-bun-migration-rep|gzmo-daemon-validation-audit-and-bun-migration-rep]] (2026-06-08)
- Uses readFileSync ×3, writeFileSync ×2, readdirSync ×1, existsSync ×2.
- High impact as Embedding Sync is massively blocked.
- Needs migration to Bun.file() / Bun.write().
