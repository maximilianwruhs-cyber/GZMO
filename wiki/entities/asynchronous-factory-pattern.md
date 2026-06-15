---
type: entity
title: asynchronous factory pattern
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# asynchronous factory pattern

Type: CONCEPT

## From [[drive-research-building-pi-coding-agent-extensions|drive-research-building-pi-coding-agent-extensions]] (2026-06-08)
- A vital architectural feature for extensions that require dynamic initialization prior to the agent entering its interactive terminal loop.
- If an extension's default export returns a Promise, the Pi agent runtime halts its startup sequence entirely, awaiting the resolution of the Promise.
