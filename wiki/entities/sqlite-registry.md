---
type: entity
title: SQLite registry
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# SQLite registry

Type: TOOL

## From [[drive-research-hermes-compression-and-bol-architecture|drive-research-hermes-compression-and-bol-architecture]] (2026-06-08)
- Used in Hermes's bifurcated local storage topology.
- Stores structured session metadata, active model configurations, token consumption metrics, and complete message histories.
- Incorporates FTS5 (Full-Text Search) indexing.
- persists action metadata (arguments, start times, and execution status) before spawning maintenance processes
- allows the remote dashboard to accurately recover and report the success or failure of the maintenance command upon reconnection
