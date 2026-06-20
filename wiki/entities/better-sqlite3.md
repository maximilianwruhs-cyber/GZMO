---
type: entity
title: better-sqlite3
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# better-sqlite3

Type: TOOL

## From [drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03](/entities/drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03.md) (2026-06-09)
- Bun integrates a native SQLite client (bun:sqlite) directly into the runtime binary.
- Because the database driver is compiled alongside the runtime, it circumvents the Node-API boundary entirely, executing queries 3 to 6 times faster than popular libraries like better-sqlite3.

## From [drive-research-bun-typescript-performance-tips-micro03](/entities/drive-research-bun-typescript-performance-tips-micro03.md) (2026-06-09)
- A popular library for SQLite.
- bun:sqlite executes queries 3 to 6 times faster than this library.
