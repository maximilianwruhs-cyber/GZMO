---
type: entity
title: AbortSignal
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# AbortSignal

Type: CONCEPT

## From [drive-research-building-pi-coding-agent-extensions](/entities/drive-research-building-pi-coding-agent-extensions.md) (2026-06-08)
- A critical architectural consideration during implementation is the handling of the AbortSignal.
- Long-running asynchronous operations within the execute block must actively consume the signal parameter.
- Allows the network layer to aggressively sever the TCP connection upon cancellation.

## From [high-performance-typescript-execution-and-architec-part1-micro06](/entities/high-performance-typescript-execution-and-architec-part1-micro06.md) (2026-06-09)
- Users can cancel ongoing model generation or tool execution by pressing the Escape key.
- Long-running asynchronous operations must actively consume the signal parameter.
- Failure to wire the AbortSignal results in orphaned promises.
- A fetch request must pass signal: ctx.signal in its options object.
- Node.js will instantly send a SIGTERM to the spawned shell when the signal is aborted.
- The AbortError is thrown by execAsync when killed.
