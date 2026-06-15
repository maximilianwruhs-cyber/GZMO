---
type: source
title: gzmo-soul-merged-new-part2-micro06
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# gzmo-soul-merged-new-part2-micro06

Ingested source summary (2026-06-10).

## Entities
- [[claude-opus|Claude Opus]] (MODEL)
- [[openclaw|OpenClaw]] (SYSTEM)
- [[executable-master-index|Executable Master Index]] (SYSTEM)
- [[model-context-protocol-mcp|Model Context Protocol (MCP)]] (PROTOCOL)
- [[gzmo-daemon|gzmo-daemon]] (SYSTEM)
- [[soul-md|SOUL.md]] (FILE)
- [[agents-md|AGENTS.md]] (FILE)
- [[firecracker|Firecracker]] (SYSTEM)
- [[obsidian-vault|Obsidian Vault]] (SYSTEM)
- [[gvisor|gVisor]] (SYSTEM)

## Relations
- SOUL.md → IS_LOCATED_IN → OpenClaw Workspace
- Executable Master Index → IS_FORBIDDEN_FROM_EDITING → SOUL.md
- Executable Master Index → UPDATES_CANONICAL_CONTRACTS_IN → Obsidian Vault
- Claude Opus → AUDITS_OUTPUTS_OF → gzmo-daemon
