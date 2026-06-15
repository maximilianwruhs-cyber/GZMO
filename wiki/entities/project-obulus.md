---
type: entity
title: Project Obulus
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Project Obulus

Type: PROJECT

## From [[obolus-micro04|obolus-micro04]] (2026-06-09)
- proposed evolutionary AI agent ecosystem
- agents compete for resources based on their energy efficiency and performance quality
- relies on mutating and evaluating agents based on their energy efficiency and performance quality
- uses an automated fitness scorer to decide which models 'survive'
- measures how much intelligence you get per watt of energy on YOUR hardware
- benchmark that measures intelligence per watt
- answers 'How smart per watt on my machine?'
- uses Intel RAPL for real energy measurement
- falls back to CPU load estimate if RAPL is not available
- defines $OBL as Energy cost in OBL, where 1 OBL = 1 Wh = 3600 J
- has task suites: math, code, factual, reasoning, full
- project structure includes CLI entry point, config, run, src (benchmark, core, simulation, integration), agents, tools, data, docs
- configuration variables include OLLAMA_URL and OBULUS_MODEL
- requires Python 3.10+, Ollama running locally, and a pulled model
