---
type: entity
title: Node.js Extension Host
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Node.js Extension Host

Type: SYSTEM

## From [[tui-framework|tui-framework]] (2026-06-08)
- Part of VSCodium's architecture.
- Can run node-pty for spawning OS processes.
- Used in VSCodium Extension development.
- Runs background tasks in VSCodium extensions.
- Uses child_process.spawn() for running tasks.
- Streams stdout/stderr to native output channels.
