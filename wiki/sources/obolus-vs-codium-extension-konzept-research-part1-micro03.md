---
type: source
title: obolus-vs-codium-extension-konzept-research-part1-micro03
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# obolus-vs-codium-extension-konzept-research-part1-micro03

Ingested source summary (2026-06-10).

## Entities
- [$OBL](/entities/obl.md) (CONCEPT)
- [AOS-Telemetrie-Engine](/entities/aos-telemetrie-engine.md) (SYSTEM)
- [Intel RAPL](/entities/intel-rapl.md) (TOOL)
- [AOS (AgenticOS)](/entities/aos-agenticos.md) (SYSTEM)
- [esbuild](/entities/esbuild.md) (TOOL)
- [Market Broker](/entities/market-broker.md) (SYSTEM)
- [Obolus-Benchmark-Runner](/entities/obolus-benchmark-runner.md) (SYSTEM)
- [Ubuntu 24.04 LTS](/entities/ubuntu-24-04-lts.md) (SYSTEM)
- [GZMO](/entities/gzmo.md) (SYSTEM)
- [FastAPI](/entities/fastapi.md) (SYSTEM)
- [Uvicorn](/entities/uvicorn.md) (SYSTEM)
- [TypeScript](/entities/typescript.md) (TOOL)
- [Chart.js](/entities/chart-js.md) (TOOL)
- [VS Code](/entities/vs-code.md) (SYSTEM)

## Relations
- AOS-Telemetrie-Engine → PART_OF → VS Code
- FastAPI → USES → Uvicorn
- FastAPI → USES → Obolus-Benchmark-Runner
- FastAPI → USES → Intel RAPL
- Market Broker → RELATED_TO → $OBL
