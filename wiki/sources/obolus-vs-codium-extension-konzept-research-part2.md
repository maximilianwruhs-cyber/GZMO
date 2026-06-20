---
type: source
title: obolus-vs-codium-extension-konzept-research-part2
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# obolus-vs-codium-extension-konzept-research-part2

Ingested source summary (2026-06-08).

## Entities
- [Node.js](/entities/node-js.md) (SYSTEM)
- [LiteParse](/entities/liteparse.md) (SYSTEM)
- [Intelligence Dashboard](/entities/intelligence-dashboard.md) (SYSTEM)
- [AOS](/entities/aos.md) (SYSTEM)
- [Chart.js](/entities/chart-js.md) (SYSTEM)
- [pgvector](/entities/pgvector.md) (SYSTEM)
- [Leaderboard](/entities/leaderboard.md) (SYSTEM)
- [Webview](/entities/webview.md) (SYSTEM)
- [Obolus VS Codium Extension](/entities/obolus-vs-codium-extension.md) (SYSTEM)
- [Reciprocal Rank Fusion (RRF)](/entities/reciprocal-rank-fusion-rrf.md) (SYSTEM)
- [FastAPI](/entities/fastapi.md) (SYSTEM)
- [TypeScript](/entities/typescript.md) (SYSTEM)
- [LM Studio](/entities/lm-studio.md) (SYSTEM)
- [BM25](/entities/bm25.md) (SYSTEM)
- [Benchmark Wizard](/entities/benchmark-wizard.md) (SYSTEM)
- [WebSocket](/entities/websocket.md) (SYSTEM)
- [gzmo-daemon](/entities/gzmo-daemon.md) (SYSTEM)
- [EAGLE](/entities/eagle.md) (SYSTEM)
- [Medusa](/entities/medusa.md) (SYSTEM)
- [Ubuntu 24.04 LTS](/entities/ubuntu-24-04-lts.md) (SYSTEM)
- [Open VSX Registry](/entities/open-vsx-registry.md) (SYSTEM)
- [tinyFolder](/entities/tinyfolder.md) (PROJECT)
- [Energy Timeline](/entities/energy-timeline.md) (SYSTEM)

## Relations
- AOS → RUNS_ON → Ubuntu 24.04 LTS
- AOS → USES → LM Studio
- AOS → USES → Obolus VS Codium Extension
- AOS → USES → gzmo-daemon
- gzmo-daemon → USES → LiteParse
- gzmo-daemon → USES → pgvector
- gzmo-daemon → USES → BM25
- gzmo-daemon → USES → Reciprocal Rank Fusion (RRF)
- Obolus VS Codium Extension → INCLUDES → Intelligence Dashboard
- Obolus VS Codium Extension → INCLUDES → Benchmark Wizard
- Obolus VS Codium Extension → INCLUDES → Leaderboard
- Obolus VS Codium Extension → INCLUDES → Energy Timeline
- Obolus VS Codium Extension → USES → TypeScript
- Obolus VS Codium Extension → USES → Webview
- Obolus VS Codium Extension → USES → FastAPI
- Obolus VS Codium Extension → USES → WebSocket
- Obolus VS Codium Extension → USES → Chart.js
- Obolus VS Codium Extension → COMPATIBLE_WITH → Open VSX Registry
- Obolus VS Codium Extension → USES → Node.js
- Obolus VS Codium Extension → FOR → AOS
