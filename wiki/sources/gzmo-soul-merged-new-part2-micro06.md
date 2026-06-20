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
- [Claude Opus](/entities/claude-opus.md) (MODEL)
- [OpenClaw](/entities/openclaw.md) (SYSTEM)
- [Executable Master Index](/entities/executable-master-index.md) (SYSTEM)
- [Model Context Protocol (MCP)](/entities/model-context-protocol-mcp.md) (PROTOCOL)
- [gzmo-daemon](/entities/gzmo-daemon.md) (SYSTEM)
- [SOUL.md](/entities/soul-md.md) (FILE)
- [AGENTS.md](/entities/agents-md.md) (FILE)
- [Firecracker](/entities/firecracker.md) (SYSTEM)
- [Obsidian Vault](/entities/obsidian-vault.md) (SYSTEM)
- [gVisor](/entities/gvisor.md) (SYSTEM)

## Relations
- SOUL.md → IS_LOCATED_IN → OpenClaw Workspace
- Executable Master Index → IS_FORBIDDEN_FROM_EDITING → SOUL.md
- Executable Master Index → UPDATES_CANONICAL_CONTRACTS_IN → Obsidian Vault
- Claude Opus → AUDITS_OUTPUTS_OF → gzmo-daemon
