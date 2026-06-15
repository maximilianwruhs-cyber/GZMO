---
type: entity
title: src/extension.ts
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# src/extension.ts

Type: SYSTEM

## From [[obolus-vs-codium-extension-konzept-research-part1-micro06|obolus-vs-codium-extension-konzept-research-part1-micro06]] (2026-06-09)
- The Entry Point for the Extension Host (Node.js).
- Initializes native elements (Status Bar) and registers the Sidebar Provider.
- Binds elements to the vscode.ExtensionContext.subscriptions Array.

## From [[obolus-vs-codium-extension-konzept-research-part1-micro07|obolus-vs-codium-extension-konzept-research-part1-micro07]] (2026-06-09)
- Serves as the Extension Host Entry Point.
- Outsources panel logic to a separate class.
- Registers the command to open the Benchmark Wizard panel.
