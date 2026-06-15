---
type: entity
title: Native API First & Abort Controller
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Native API First & Abort Controller

Type: CONCEPT

## From [[obolus-vs-codium-extension-konzept-research-part1-micro05|obolus-vs-codium-extension-konzept-research-part1-micro05]] (2026-06-09)
- Native Node 18 `fetch` API must be used for REST calls.
- Long-running tasks must be encapsulated in `vscode.window.withProgress`.
- Supports CancellationToken bound to an AbortController for task cancellation.
