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
- [[typescript|TypeScript]] (CONCEPT)
- [[node-js|Node.js]] (SYSTEM)
- [[extensioncontext|ExtensionContext]] (CONCEPT)
- [[mariozechner-pi-tui|@mariozechner/pi-tui]] (TOOL)
- [[vers-api|Vers API]] (SYSTEM)
- [[git|git]] (TOOL)
- [[ci-cd-pipelines|CI/CD pipelines]] (SYSTEM)
- [[mariozechner-pi-coding-agent|@mariozechner/pi-coding-agent]] (SYSTEM)
- [[mario-zechner|Mario Zechner]] (PERSON)
- [[defaultresourceloader|DefaultResourceLoader]] (SYSTEM)
- [[jiti|jiti]] (TOOL)
- [[vers-vm-extension|Vers VM extension]] (PROJECT)
- [[extensionapi|ExtensionAPI]] (CONCEPT)
- [[json-rpc|JSON-RPC]] (CONCEPT)
- [[pi-mem|pi-mem]] (TOOL)
- [[db0-ai-pi|@db0-ai/pi]] (TOOL)
- [[tmux|tmux]] (SYSTEM)
- [[blessed|Blessed]] (TOOL)
- [[claude-code|Claude Code]] (SYSTEM)
- [[stringenum|StringEnum]] (TOOL)
- [[termux|Termux]] (SYSTEM)
- [[slack-bots|Slack bots]] (SYSTEM)
- [[ink|Ink]] (TOOL)
- [[typebox|TypeBox]] (TOOL)
- [[npm|npm]] (TOOL)
- [[emacs|Emacs]] (TOOL)
- [[sqlite|SQLite]] (SYSTEM)

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
