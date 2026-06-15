---
type: entity
title: 'AOS: Open Benchmark Wizard'
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# AOS: Open Benchmark Wizard

Type: TOOL

## From [[obolus-vs-codium-extension-konzept-research-part1-micro06|obolus-vs-codium-extension-konzept-research-part1-micro06]] (2026-06-09)
- Connected to the FastAPI backend.
- Local eGPU benchmark can take minutes.
- Has an HTML wizard with a loading bar.
- IPC message is captured and forwarded to the backend.
- Located in src/panels/BenchmarkWizardPanel.ts.
- Has a 'Run' button in the webview panel.
- Can be cancelled by the user.
- Requires a clean scaffolding for development.
- A command registered in package.json.
- Opens the Benchmark Wizard Panel.
