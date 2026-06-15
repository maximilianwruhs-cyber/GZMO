---
type: entity
title: V8 engine
created: 2026-06-09
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# V8 engine

Type: SYSTEM

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- N-API introduces significant serialization and deserialization overhead as data crosses the boundary between the V8 engine and the C++ runtime.
- Legacy packages often make direct assumptions about V8 engine internals.

## From [[drive-research-bun-typescript-performance-tips-micro02|drive-research-bun-typescript-performance-tips-micro02]] (2026-06-09)
- The foundation of Node.js and Deno.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Data crosses the boundary between V8 and C++ runtime with overhead.
- Bun's FFI converts values between JavaScript types and native memory types.
- Assumptions about V8 engine internals are made by some legacy packages.
- Garbage collection cycles are non-deterministic.

## From [[high-performance-typescript-execution-and-architec-part1-micro02|high-performance-typescript-execution-and-architec-part1-micro02]] (2026-06-09)
- The foundation of Node.js and Deno.
- Bun does not utilize the V8 engine.
