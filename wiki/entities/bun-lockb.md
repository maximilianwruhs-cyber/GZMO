---
type: entity
title: bun.lockb
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# bun.lockb

Type: TOOL

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- The resulting dependency tree is locked using bun.lockb, a proprietary binary lockfile format.
- This binary format parses significantly faster than traditional JSON or YAML lockfiles.
- Implements a highly optimized Foreign Function Interface (bun:ffi) natively within its core.
- Utilizes an embedded instance of TinyCC for JIT compilation of C bindings.
- Integrates a native SQLite client (bun:sqlite) directly into the runtime binary.
- Has a native HTTP server initialized via Bun.serve().
- Incorporates a robust internal DNS cache.
- Has experimental Bun.dns.prefetch() API.
- Uses bunfig.toml for environmental control.
- Has a proprietary binary lockfile format (bun.lockb).
- bun test operates as a native, Jest-compatible test runner.
- Processes an estimated 180,000 requests per second in synthetic HTTP benchmarks.
- Has a cold start latency of 8ms to 15ms.
- Achieved 95% to 98% API compatibility with the Node.js ecosystem.
- Official oven/bun Docker image is roughly 450MB.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Bun's proprietary binary lockfile format.
- Parses significantly faster than traditional JSON or YAML lockfiles.
