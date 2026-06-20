---
type: entity
title: Node.js module cache
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Node.js module cache

Type: SYSTEM

## From [drive-research-building-pi-coding-agent-extensions](/entities/drive-research-building-pi-coding-agent-extensions.md) (2026-06-08)
- When a reload is initiated, the Node.js module cache for the extensions is completely flushed.
- Extensions operate with native Node.js core modules.
- The Pi agent automatically intercepts and resolves imports from the node_modules/ directory.
- The Pi package installation process executes npm install --omit=dev by default.
- Provides the OS module for memory usage statistics.
- Utilized for non-blocking background polling intervals.
- Its module caching affects extension updates.

## From [high-performance-typescript-execution-and-architec-part1-micro07](/entities/high-performance-typescript-execution-and-architec-part1-micro07.md) (2026-06-09)
- Maintained for exceptionally fast performance.
- Flushed during the /reload process.
