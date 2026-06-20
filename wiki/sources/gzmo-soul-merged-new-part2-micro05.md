---
type: source
title: gzmo-soul-merged-new-part2-micro05
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# gzmo-soul-merged-new-part2-micro05

Ingested source summary (2026-06-09).

## Entities
- [CodeAct](/entities/codeact.md) (CONCEPT)
- [Specialist](/entities/specialist.md) (SYSTEM)
- [[proc-pid-stat|/proc/[PID]/stat]] (SYSTEM)
- [PowerShell](/entities/powershell.md) (TOOL)
- [PARA](/entities/para.md) (CONCEPT)
- [Winget](/entities/winget.md) (TOOL)
- [procfs](/entities/procfs.md) (SYSTEM)
- [Scoop](/entities/scoop.md) (TOOL)
- [GZMO](/entities/gzmo.md) (SYSTEM)
- [Agentic RAG](/entities/agentic-rag.md) (CONCEPT)
- [Multi-Agent Systems](/entities/multi-agent-systems.md) (CONCEPT)
- [Librarian Agent](/entities/librarian-agent.md) (SYSTEM)
- [[proc-pid-statm|/proc/[PID]/statm]] (SYSTEM)
- [DeepWideSearch](/entities/deepwidesearch.md) (TOOL)
- [Johnny Decimal](/entities/johnny-decimal.md) (CONCEPT)
- [Linux](/entities/linux.md) (SYSTEM)
- [Fact-Checker](/entities/fact-checker.md) (SYSTEM)
- [Editor/Critic](/entities/editor-critic.md) (SYSTEM)
- [Model Context Protocol (MCP)](/entities/model-context-protocol-mcp.md) (CONCEPT)
- [Chocolatey](/entities/chocolatey.md) (TOOL)
- [OpenClaw](/entities/openclaw.md) (SYSTEM)
- [SMAS](/entities/smas.md) (SYSTEM)
- [pidfd API](/entities/pidfd-api.md) (TOOL)

## Relations
- GZMO → PART_OF → OpenClaw
- Librarian Agent → USES → Agentic RAG
- GZMO → USES → Linux
- GZMO → USES → OpenClaw
- Librarian Agent → PART_OF → Multi-Agent Systems
- Specialist → PART_OF → Multi-Agent Systems
- Fact-Checker → PART_OF → Multi-Agent Systems
- Editor/Critic → PART_OF → Multi-Agent Systems
- Winget → USES → Linux
- Chocolatey → USES → Linux
- Scoop → USES → Linux
- PowerShell → USES → Linux
- pidfd API → USES → Linux
- CodeAct → USES → Multi-Agent Systems
- SMAS → USES → Multi-Agent Systems
- Model Context Protocol (MCP) → USES → Multi-Agent Systems
- GZMO → USES → procfs
- Linux → USES → procfs
- /proc/[PID]/stat → PART_OF → procfs
- /proc/[PID]/statm → PART_OF → procfs
