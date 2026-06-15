---
type: entity
title: Extension Host
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Extension Host

Type: SYSTEM

## From [[obolus-vs-codium-extension-konzept-research-part1-micro05|obolus-vs-codium-extension-konzept-research-part1-micro05]] (2026-06-09)
- Node.js process where the extension runs.
- Handles WebSocket connection and throttling.
- Can be frozen by unfiltered data streams.

## From [[obolus-vs-codium-extension-konzept-research-part1-micro06|obolus-vs-codium-extension-konzept-research-part1-micro06]] (2026-06-09)
- Runs on Node.js 18+ since VSCodium / VS Code 1.79.
- Receives IPC messages from the Wizard.
- Is the context for the BenchmarkWizardPanel.
- Is the context for the Entry Point (Node.js).
