---
type: entity
title: Bun.write
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Bun.write

Type: TOOL

## From [refactoring-gzmo-daemon-for-native-bun-high-perfor](/entities/refactoring-gzmo-daemon-for-native-bun-high-perfor.md) (2026-06-08)
- Will replace fs and fs.promises.
- Leverages io_uring on Linux.
- Used for crystallization.
- Used to translate resulting insights.
- Used for fire-and-forget persistence.

## From [drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03](/entities/drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03.md) (2026-06-09)
- Bun dismantles this pipeline by implementing the Bun.file() and Bun.write() APIs.
- Bun.write() can operate up to 10 times faster than the equivalent fs.writeFileSync() in Node.js.

## From [drive-research-bun-typescript-performance-tips-micro03](/entities/drive-research-bun-typescript-performance-tips-micro03.md) (2026-06-09)
- Implemented by Bun.
- Can operate up to 10 times faster than Node.js fs.writeFileSync().
