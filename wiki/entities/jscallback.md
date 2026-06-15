---
type: entity
title: JSCallback
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# JSCallback

Type: CONCEPT

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Bun's FFI system supports JSCallback.
- Allows native C or Rust code to asynchronously invoke TypeScript functions.
- Passing JSCallback.prototype.ptr directly to the native function reduces invocation overhead.
