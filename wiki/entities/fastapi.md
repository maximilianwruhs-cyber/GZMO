---
type: entity
title: FastAPI
created: 2026-06-08
updated: 2026-06-10
sources: 6
tags: []
status: draft
gzmo_synthetic: true
---






# FastAPI

Type: SYSTEM

## From [[obolus-vs-codium-extension-konzept-research-part2|obolus-vs-codium-extension-konzept-research-part2]] (2026-06-08)
- Python-Backend (AOS FastAPI Gateway on localhost:8000)
- FastAPI WebSocket implementation.

## From [[obolus-vs-codium-extension-konzept-research-part1-micro04|obolus-vs-codium-extension-konzept-research-part1-micro04]] (2026-06-09)
- A web framework.
- Used for creating WebSockets.
- Can be run with `fastapi dev`.

## From [[obolus-vs-codium-extension-konzept-research-part1-micro05|obolus-vs-codium-extension-konzept-research-part1-micro05]] (2026-06-09)
- Trademark owned by @tiangolo.
- Backend for telemetry data.

## From [[obolus-vs-codium-extension-konzept-research-part1-micro06|obolus-vs-codium-extension-konzept-research-part1-micro06]] (2026-06-09)
- Benchmark-Wizard is connected to its backend.
- The native fetch call is made to its backend.
- The Obolus-runner must be killed by the backend upon connection abort.

## From [[obolus-vs-codium-extension-konzept-research-part1-micro07|obolus-vs-codium-extension-konzept-research-part1-micro07]] (2026-06-09)
- A backend that the Benchmark Wizard can make REST calls to.
- Mentioned as a target for Phase 4.

## From [[obolus-vs-codium-extension-konzept-research-part1-micro03|obolus-vs-codium-extension-konzept-research-part1-micro03]] (2026-06-10)
- Used as the backend service (AOS Gateway).
- Runs under Uvicorn.
- Provides native WebSocket implementation for /ws/telemetry.
