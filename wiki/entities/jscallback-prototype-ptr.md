---
type: entity
title: JSCallback.prototype.ptr
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# JSCallback.prototype.ptr

Type: TOOL

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Bun's FFI system also supports JSCallback, allowing native C or Rust code to asynchronously invoke TypeScript functions.
- A critical optimization tip is to pass JSCallback.prototype.ptr directly to the native function rather than the JSCallback instance object itself, yielding a measurable reduction in invocation overhead.
