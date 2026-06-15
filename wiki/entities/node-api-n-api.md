---
type: entity
title: Node-API (N-API)
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# Node-API (N-API)

Type: TOOL

## From [[drive-research-license-and-native-binding-analysis|drive-research-license-and-native-binding-analysis]] (2026-06-08)
- An abstraction framework intended to insulate native C++ addons from changes in the underlying JavaScript engine.
- Bun implements a compatibility layer for N-API.
- Pre-built Node-API (N-API) binaries are downloaded by node-pre-gyp.

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Historically, the Node.js ecosystem relied on Node-API (N-API) or node-gyp native addons.
- N-API introduces significant serialization and deserialization overhead.
- Benchmark data shows bun:ffi executes function calls 2 to 6 times faster than standard Node.js FFI implementations operating via Node-API.
- Bun circumvents the Node-API boundary entirely with its native SQLite client.
- Bun's N-API translation layer cannot safely abstract or mimic assumptions made by legacy packages.

## From [[drive-research-bun-file-parsing-dependency-shortlist-micro01|drive-research-bun-file-parsing-dependency-shortlist-micro01]] (2026-06-09)
- interface used by native libraries to interact with the runtime

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Introduces significant serialization and deserialization overhead.
- Used by Node.js ecosystem for native addons.
- Bun's FFI circumvents the Node-API boundary.
