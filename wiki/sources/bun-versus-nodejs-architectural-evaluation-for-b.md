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
- [Google's V8 engine](/entities/google-s-v8-engine.md) (SYSTEM)
- [Obsidian Vault](/entities/obsidian-vault.md) (CONCEPT)
- [Zig](/entities/zig.md) (TOOL)
- [Node.js](/entities/node-js.md) (SYSTEM)
- [Chokidar](/entities/chokidar.md) (TOOL)
- [TypeScript](/entities/typescript.md) (CONCEPT)
- [libuv](/entities/libuv.md) (SYSTEM)
- [kqueue](/entities/kqueue.md) (SYSTEM)
- [inotify](/entities/inotify.md) (SYSTEM)
- [mimalloc](/entities/mimalloc.md) (TOOL)
- [Apple's JavaScriptCore engine](/entities/apple-s-javascriptcore-engine.md) (SYSTEM)
- [SWC (Speedy Web Compiler)](/entities/swc-speedy-web-compiler.md) (TOOL)
- [Amaro loader](/entities/amaro-loader.md) (TOOL)
- [Bun](/entities/bun.md) (SYSTEM)

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
