---
type: entity
title: Foreign Function Interface (FFI)
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Foreign Function Interface (FFI)

Type: CONCEPT

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02]] (2026-06-09)
- Bun supports high-performance Foreign Function Interfaces.
- Referred to as 'bun:ffi'.

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Bun resolves this bottleneck by implementing a highly optimized Foreign Function Interface (bun:ffi) natively within its core.
- The JIT-Compiled FFI Advantage.
- Benchmark data consistently demonstrates that bun:ffi executes function calls 2 to 6 times faster than standard Node.js FFI implementations operating via Node-API.
- Operating outside the bounds of a garbage-collected engine requires meticulous memory management.
- When executing FFI calls, data conversion requires deliberate finesse.
- The primary peril of FFI in a garbage-collected language is memory leakage.
- Bun's FFI system also supports JSCallback.
- Utilizing the JIT-compiled bun:ffi for memory-safe native execution across language boundaries.

## From [[high-performance-typescript-execution-and-architec-part1-micro02|high-performance-typescript-execution-and-architec-part1-micro02]] (2026-06-09)
- Advanced technique for executing TypeScript at peak efficiency.
- Bun supports FFI via bun:ffi.
