---
type: entity
title: Macros
created: 2026-06-09
updated: 2026-06-09
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---






# Macros

Type: CONCEPT

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02]] (2026-06-09)
- A native metaprogramming construct in Bun.
- Shifts runtime execution to build time.
- Executed during the build process by Bun's native transpiler.
- Returned values are inlined into the compiled bundle as AST nodes.
- Execute in parallel using JavaScript worker threads.
- Output must be serializable (JSON, TypedArrays, Response Objects).
- Cannot accept runtime-dynamic arguments.
- Enable zero-latency data initialization and advanced dead code elimination.

## From [[drive-research-bun-typescript-performance-tips-micro02|drive-research-bun-typescript-performance-tips-micro02]] (2026-06-09)
- A native metaprogramming construct in Bun.
- Allows shifting runtime execution to build time.
- Executed during the transpiler's AST visiting phase.
- Can be imported using ECMAScript import attributes.
- Output must be serializable.
- Can be used for zero-latency data initialization and dead code elimination.

## From [[drive-research-bun-typescript-performance-tips-micro04|drive-research-bun-typescript-performance-tips-micro04]] (2026-06-09)
- A feature of Bun.
- Related documentation available on Mintlify.

## From [[high-performance-typescript-execution-and-architec-part1-micro02|high-performance-typescript-execution-and-architec-part1-micro02]] (2026-06-09)
- A native metaprogramming construct in Bun.
- Shifts runtime execution to build time.
- Executed during the transpiler's AST visiting phase.
- Output must be serializable (JSON, TypedArrays, Response Objects).
- Cannot accept runtime-dynamic arguments.
- Enable zero-latency data initialization and advanced dead code elimination.
- Enhance security by executing privileged operations at build time.

## From [[high-performance-typescript-execution-and-architec-part1-micro04|high-performance-typescript-execution-and-architec-part1-micro04]] (2026-06-09)
- Bun has a Macros feature.
- Mentioned as a cool Bun feature.
