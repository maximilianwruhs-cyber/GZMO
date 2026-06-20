---
type: entity
title: SidebarProvider
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# SidebarProvider

Type: SYSTEM

## From [obolus-vs-codium-extension-konzept-research-part1-micro05](/entities/obolus-vs-codium-extension-konzept-research-part1-micro05.md) (2026-06-09)
- Class responsible for the sidebar view.
- Integrates TelemetryService.
- Resolves webview and attaches it to TelemetryService.

## From [obolus-vs-codium-extension-konzept-research-part1-micro06](/entities/obolus-vs-codium-extension-konzept-research-part1-micro06.md) (2026-06-09)
- Class that generates HTML for the sidebar.
- Implements strict Content Security Policy (CSP).
- Uses asWebviewUri to securely load local scripts.
- Is registered as a webview view provider.
