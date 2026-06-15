---
type: entity
title: State Management
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# State Management

Type: CONCEPT

## From [[obolus-vs-codium-extension-konzept-research-part1-micro05|obolus-vs-codium-extension-konzept-research-part1-micro05]] (2026-06-09)
- Webview Panels configured with `retainContextWhenHidden: false`.
- Webview frontends must implement `acquireVsCodeApi().setState()` and `getState()` for persistence.
