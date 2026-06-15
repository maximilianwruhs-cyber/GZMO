---
type: entity
title: TypedArray
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# TypedArray

Type: CONCEPT

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Engineers should pass JavaScript TypedArray objects (such as Uint8Array or Float32Array) directly to the native function.
- Bun automatically extracts the underlying memory pointer of the array buffer and passes it to the C Application Binary Interface (ABI), enabling true zero-copy data manipulation.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Engineers should pass JavaScript TypedArray objects directly to native functions for maximum throughput.
- Bun automatically extracts the underlying memory pointer of the array buffer.
