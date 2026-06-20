---
type: entity
title: Benchmark Wizard
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Benchmark Wizard

Type: SYSTEM

## From [obolus-vs-codium-extension-konzept-research-part2](/entities/obolus-vs-codium-extension-konzept-research-part2.md) (2026-06-08)
- Webview Panel: Benchmark-Wizard
- Modell auswählen, Suite wählen (math, code, full), Fortschrittsanzeige via WebSocket, Ergebnis: Fitness-Score, Energy, z-Score.

## From [obolus-vs-codium-extension-konzept-research-part1-micro05](/entities/obolus-vs-codium-extension-konzept-research-part1-micro05.md) (2026-06-09)
- Webview Panel.
- Interactive form for GZMO-routing override.
- Used to start Obolus-Suites.

## From [obolus-vs-codium-extension-konzept-research-part1-micro07](/entities/obolus-vs-codium-extension-konzept-research-part1-micro07.md) (2026-06-09)
- Implemented using vscode.window.createWebviewPanel.
- Opens a new, full-fledged tab in the main editor area.
- Requires State Management via getState/setState to prevent benchmark progress loss.
