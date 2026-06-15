---
type: entity
title: Intelligence Dashboard
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Intelligence Dashboard

Type: SYSTEM

## From [[obolus-vs-codium-extension-konzept-research-part2|obolus-vs-codium-extension-konzept-research-part2]] (2026-06-08)
- A 5-pillar IDE extension (built with TypeScript, WebSockets, and a FastAPI backend) that tracks the active model, its energy consumption (Joules per request), and ranks models on a leaderboard using Z-Scores.
- Sidebar: Intelligence Dashboard
- Registriert als eigener Activity Bar Icon. Zeigt Gateway-Status, aktuelles Modell, Energy, z-Score, $OBL-Preis, Quick Actions (Run Benchmark, Switch Model, Open Leaderboard), Recent Evaluations.

## From [[obolus-vs-codium-extension-konzept-research-part1-micro05|obolus-vs-codium-extension-konzept-research-part1-micro05]] (2026-06-09)
- WebviewViewProvider in the Activity Bar.
- Shows live status of $OBL-price and GZMO-autonomy.
