---
type: entity
title: Node.js 24
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Node.js 24

Type: SYSTEM

## From [drive-research-bun-typescript-performance-tips-micro03](/entities/drive-research-bun-typescript-performance-tips-micro03.md) (2026-06-09)
- Ecosystem historically relied on Node-API (N-API) or node-gyp.
- FFI implementations operate via Node-API.
- Traditionally executes file reads via standard C++ abstractions.
- fs.writeFileSync() is an equivalent to Bun.write().
- Has a baseline cold start latency of 60ms to 120ms.
- Container footprint is approximately 180MB.
- Has an estimated 65,000 requests per second in synthetic benchmarks.
