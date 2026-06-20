---
type: source
title: the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro07
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro07

Ingested source summary (2026-06-09).

## Entities
- [Puppeteer (Headless Browser)](/entities/puppeteer-headless-browser.md) (TOOL)
- [ReAct (Reason + Act)](/entities/react-reason-act.md) (CONCEPT)
- [SOUL.md](/entities/soul-md.md) (CONCEPT)
- [AGENTS.md](/entities/agents-md.md) (CONCEPT)
- [sqlite-vec](/entities/sqlite-vec.md) (TOOL)
- [Critic-Node](/entities/critic-node.md) (CONCEPT)
- [OpenClaw-Framework](/entities/openclaw-framework.md) (FRAMEWORK)
- [Bootstrap Context Files](/entities/bootstrap-context-files.md) (CONCEPT)
- [Fact-Checker Agent (v3)](/entities/fact-checker-agent-v3.md) (SYSTEM)
- [SHARP taste gate](/entities/sharp-taste-gate.md) (CONCEPT)

## Relations
- Fact-Checker Agent (v3) → PART_OF → OpenClaw-Framework
- Fact-Checker Agent (v3) → USES → ReAct (Reason + Act)
- Fact-Checker Agent (v3) → USES → Bootstrap Context Files
- Bootstrap Context Files → RELATED_TO → SOUL.md
- Bootstrap Context Files → RELATED_TO → AGENTS.md
- ReAct (Reason + Act) → USES → sqlite-vec
- ReAct (Reason + Act) → USES → Puppeteer (Headless Browser)
- ReAct (Reason + Act) → USES → Critic-Node
