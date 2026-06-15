---
type: source
title: bun-versus-nodejs-architectural-evaluation-for-b
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# bun-versus-nodejs-architectural-evaluation-for-b

Ingested source summary (2026-06-08).

## Entities
- [[google-s-v8-engine|Google's V8 engine]] (SYSTEM)
- [[obsidian-vault|Obsidian Vault]] (CONCEPT)
- [[zig|Zig]] (TOOL)
- [[node-js|Node.js]] (SYSTEM)
- [[chokidar|Chokidar]] (TOOL)
- [[typescript|TypeScript]] (CONCEPT)
- [[libuv|libuv]] (SYSTEM)
- [[kqueue|kqueue]] (SYSTEM)
- [[inotify|inotify]] (SYSTEM)
- [[mimalloc|mimalloc]] (TOOL)
- [[apple-s-javascriptcore-engine|Apple's JavaScriptCore engine]] (SYSTEM)
- [[swc-speedy-web-compiler|SWC (Speedy Web Compiler)]] (TOOL)
- [[amaro-loader|Amaro loader]] (TOOL)
- [[bun|Bun]] (SYSTEM)

## Relations
- Node.js → USES → Google's V8 engine
- Bun → USES → Apple's JavaScriptCore engine
- Bun → USES → Zig
- Node.js → PART_OF → libuv
- Bun → USES → kqueue
- Bun → USES → inotify
- Bun → USES → mimalloc
- Node.js → USES → Amaro loader
- Amaro loader → USES → SWC (Speedy Web Compiler)
- Chokidar → RELATED_TO → Node.js
- Obsidian Vault → RELATED_TO → TypeScript
