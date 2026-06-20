---
type: source
title: architectural-framework-and-development-of-pi-codi
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# architectural-framework-and-development-of-pi-codi

Ingested source summary (2026-06-08).

## Entities
- [TypeScript](/entities/typescript.md) (CONCEPT)
- [Node.js](/entities/node-js.md) (SYSTEM)
- [ExtensionContext](/entities/extensioncontext.md) (CONCEPT)
- [@mariozechner/pi-tui](/entities/mariozechner-pi-tui.md) (TOOL)
- [Vers API](/entities/vers-api.md) (SYSTEM)
- [git](/entities/git.md) (TOOL)
- [CI/CD pipelines](/entities/ci-cd-pipelines.md) (SYSTEM)
- [@mariozechner/pi-coding-agent](/entities/mariozechner-pi-coding-agent.md) (SYSTEM)
- [Mario Zechner](/entities/mario-zechner.md) (PERSON)
- [DefaultResourceLoader](/entities/defaultresourceloader.md) (SYSTEM)
- [jiti](/entities/jiti.md) (TOOL)
- [Vers VM extension](/entities/vers-vm-extension.md) (PROJECT)
- [ExtensionAPI](/entities/extensionapi.md) (CONCEPT)
- [JSON-RPC](/entities/json-rpc.md) (CONCEPT)
- [pi-mem](/entities/pi-mem.md) (TOOL)
- [@db0-ai/pi](/entities/db0-ai-pi.md) (TOOL)
- [tmux](/entities/tmux.md) (SYSTEM)
- [Blessed](/entities/blessed.md) (TOOL)
- [Claude Code](/entities/claude-code.md) (SYSTEM)
- [StringEnum](/entities/stringenum.md) (TOOL)
- [Termux](/entities/termux.md) (SYSTEM)
- [Slack bots](/entities/slack-bots.md) (SYSTEM)
- [Ink](/entities/ink.md) (TOOL)
- [TypeBox](/entities/typebox.md) (TOOL)
- [npm](/entities/npm.md) (TOOL)
- [Emacs](/entities/emacs.md) (TOOL)
- [SQLite](/entities/sqlite.md) (SYSTEM)

## Relations
- @mariozechner/pi-coding-agent → AUTHORED_BY → Mario Zechner
- @mariozechner/pi-coding-agent → RELATED_TO → Claude Code
- @mariozechner/pi-coding-agent → USES → Node.js
- @mariozechner/pi-coding-agent → USES → jiti
- @mariozechner/pi-coding-agent → USES → TypeScript
- @mariozechner/pi-coding-agent → USES → ExtensionAPI
- @mariozechner/pi-coding-agent → USES → DefaultResourceLoader
- @mariozechner/pi-coding-agent → USES → TypeBox
- @mariozechner/pi-coding-agent → USES → StringEnum
- @mariozechner/pi-coding-agent → USES → ExtensionContext
- @mariozechner/pi-coding-agent → USES → @mariozechner/pi-tui
- @mariozechner/pi-coding-agent → RELATED_TO → Ink
- @mariozechner/pi-coding-agent → RELATED_TO → Blessed
- @mariozechner/pi-coding-agent → USES → tmux
- @mariozechner/pi-coding-agent → USES → Termux
- @mariozechner/pi-coding-agent → USES → Vers VM extension
- Vers VM extension → USES → Vers API
- @mariozechner/pi-coding-agent → USES → npm
- @mariozechner/pi-coding-agent → USES → git
- @mariozechner/pi-coding-agent → USES → JSON-RPC
- @mariozechner/pi-coding-agent → USES → Emacs
- @mariozechner/pi-coding-agent → USES → Slack bots
- @mariozechner/pi-coding-agent → USES → CI/CD pipelines
- @mariozechner/pi-coding-agent → USES → pi-mem
- @mariozechner/pi-coding-agent → USES → @db0-ai/pi
- pi-mem → USES → SQLite
- @db0-ai/pi → USES → SQLite
