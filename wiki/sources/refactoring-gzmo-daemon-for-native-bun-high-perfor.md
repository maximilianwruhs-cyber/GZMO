---
type: source
title: refactoring-gzmo-daemon-for-native-bun-high-perfor
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# refactoring-gzmo-daemon-for-native-bun-high-perfor

Ingested source summary (2026-06-08).

## Entities
- [bunfig.toml](/entities/bunfig-toml.md) (TOOL)
- [fs.promises](/entities/fs-promises.md) (TOOL)
- [NotebookLM](/entities/notebooklm.md) (TOOL)
- [GZMO Daemon](/entities/gzmo-daemon.md) (PROJECT)
- [io_uring](/entities/io-uring.md) (CONCEPT)
- [smol = true](/entities/smol-true.md) (CONCEPT)
- [Bun.peek](/entities/bun-peek.md) (TOOL)
- [Google Takeout](/entities/google-takeout.md) (TOOL)
- [Edge Node](/entities/edge-node.md) (SYSTEM)
- [Bun.file](/entities/bun-file.md) (TOOL)
- [JavaScriptCore](/entities/javascriptcore.md) (SYSTEM)
- [Node.js](/entities/node-js.md) (SYSTEM)
- [Mastering Bun: High-Performance TypeScript Execution and Architecture](/entities/mastering-bun-high-performance-typescript-execution-and-architecture.md) (BOOK)
- [Bun.write](/entities/bun-write.md) (TOOL)

## Relations
- GZMO Daemon → USES → Bun.peek
- GZMO Daemon → USES → Node.js
- GZMO Daemon → PART_OF → Edge Node
- NotebookLM → AUTHORED_BY → Mastering Bun: High-Performance TypeScript Execution and Architecture
- GZMO Daemon → USES → fs.promises
- Bun.write → RELATED_TO → fs.promises
- Bun.file → RELATED_TO → fs.promises
- Bun.write → USES → io_uring
- Bun.file → USES → io_uring
- bunfig.toml → USES → JavaScriptCore
- smol = true → PART_OF → bunfig.toml
- Edge Node → USES → Bun.peek
- NotebookLM → USES → Google Takeout
