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
- [[obl|$OBL]] (CONCEPT)
- [[aos-telemetrie-engine|AOS-Telemetrie-Engine]] (SYSTEM)
- [[intel-rapl|Intel RAPL]] (TOOL)
- [[aos-agenticos|AOS (AgenticOS)]] (SYSTEM)
- [[esbuild|esbuild]] (TOOL)
- [[market-broker|Market Broker]] (SYSTEM)
- [[obolus-benchmark-runner|Obolus-Benchmark-Runner]] (SYSTEM)
- [[ubuntu-24-04-lts|Ubuntu 24.04 LTS]] (SYSTEM)
- [[gzmo|GZMO]] (SYSTEM)
- [[fastapi|FastAPI]] (SYSTEM)
- [[uvicorn|Uvicorn]] (SYSTEM)
- [[typescript|TypeScript]] (TOOL)
- [[chart-js|Chart.js]] (TOOL)
- [[vs-code|VS Code]] (SYSTEM)

## Relations
- AOS-Telemetrie-Engine → PART_OF → VS Code
- FastAPI → USES → Uvicorn
- FastAPI → USES → Obolus-Benchmark-Runner
- FastAPI → USES → Intel RAPL
- Market Broker → RELATED_TO → $OBL
