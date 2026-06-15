---
type: entity
title: Content Security Policy
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Content Security Policy

Type: CONCEPT

## From [[obolus-vs-codium-extension-konzept-research-part1-micro05|obolus-vs-codium-extension-konzept-research-part1-micro05]] (2026-06-09)
- Strict CSP tag required for every Webview.
- Scripts (`script-src`) allowed only with dynamically generated nonce.
- Local assets must be loaded via `webview.asWebviewUri()`.
