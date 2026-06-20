---
type: entity
title: Bun.peek.status()
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Bun.peek.status()

Type: TOOL

## From [drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02](/entities/drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02.md) (2026-06-09)
- Returns a string literal representing the internal state of a promise: 'fulfilled', 'pending', or 'rejected'.
- Allows for complex state monitoring and branching logic without consuming the result.
- An advanced, low-level utility for latency-critical code.
- Allows the runtime to inspect the status and extract the value of a promise synchronously.
- Bypasses the event loop entirely.
- Primary use case is within high-throughput caching layers or custom HTTP routers.
- Can be used to check for in-memory cache hits.

## From [high-performance-typescript-execution-and-architec-part1-micro02](/entities/high-performance-typescript-execution-and-architec-part1-micro02.md) (2026-06-09)
- Returns a string literal representing the internal state of a promise ('fulfilled', 'pending', or 'rejected').
- Allows for state monitoring without consuming the result.
- A low-level utility for latency-critical code.
- Inspects the status and extracts the value of a promise synchronously.
- Bypasses the event loop.
- Can return the value, the pending promise, or the error object.
- Does not mark rejected promises as handled.
