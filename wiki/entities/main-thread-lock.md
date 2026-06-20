---
type: entity
title: Main-Thread-Lock
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Main-Thread-Lock

Type: CONCEPT

## From [obolus-vs-codium-extension-konzept-research-part1-micro05](/entities/obolus-vs-codium-extension-konzept-research-part1-micro05.md) (2026-06-09)
- Architectural risk in asynchronous communication.
- Occurs when FastAPI server fires hundreds of events per second unfiltered.
- Solution is 10Hz throttling/debouncing.
