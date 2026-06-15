---
type: entity
title: TinyCC
created: 2026-06-09
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# TinyCC

Type: TOOL

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Bun relies on an embedded instance of TinyCC (a remarkably small and fast C compiler).
- At runtime, Bun dynamically generates and Just-In-Time (JIT) compiles bespoke C bindings that perfectly match the required function signatures.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- A remarkably small and fast C compiler.
- Bun relies on an embedded instance of TinyCC for bun:ffi.

## From [[high-performance-typescript-execution-and-architec-part1-micro03|high-performance-typescript-execution-and-architec-part1-micro03]] (2026-06-10)
- A remarkably small and fast C compiler.
- Embedded within Bun to JIT compile bespoke C bindings.
