---
type: source
title: drive-research-how-could-we-blueprint-an-idea
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-how-could-we-blueprint-an-idea

Ingested source summary (2026-06-08).

## Entities
- [Formal Conversational Protocol (FCoP)](/entities/formal-conversational-protocol-fcop.md) (CONCEPT)
- [Sovereign Software Factory](/entities/sovereign-software-factory.md) (SYSTEM)
- [TASK](/entities/task.md) (CONCEPT)
- [Defender Agent](/entities/defender-agent.md) (CONCEPT)
- [Prosecutor Agent](/entities/prosecutor-agent.md) (CONCEPT)
- [Obsidian](/entities/obsidian.md) (TOOL)
- [Prosecutor-Defender-Umpire](/entities/prosecutor-defender-umpire.md) (CONCEPT)
- [Rust-based TUI](/entities/rust-based-tui.md) (TOOL)
- [ISSUE](/entities/issue.md) (CONCEPT)
- [UNBOUND](/entities/unbound.md) (CONCEPT)
- [Umpire Agent](/entities/umpire-agent.md) (CONCEPT)
- [VSCodium](/entities/vscodium.md) (TOOL)
- [Active Conversation Protocol (ACP)](/entities/active-conversation-protocol-acp.md) (CONCEPT)
- [REPORT](/entities/report.md) (CONCEPT)

## Relations
- Sovereign Software Factory → PART_OF → Obsidian
- Sovereign Software Factory → PART_OF → VSCodium
- TASK → PART_OF → Sovereign Software Factory
- REPORT → PART_OF → Sovereign Software Factory
- ISSUE → PART_OF → Sovereign Software Factory
- ISSUE → RELATED_TO → UNBOUND
- Prosecutor Agent → PART_OF → Prosecutor-Defender-Umpire
- Defender Agent → PART_OF → Prosecutor-Defender-Umpire
- Umpire Agent → PART_OF → Prosecutor-Defender-Umpire
- Rust-based TUI → USES → TASK
- Rust-based TUI → USES → REPORT
- Rust-based TUI → USES → ISSUE
