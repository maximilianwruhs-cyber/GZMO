---
type: entity
title: file system buffer
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# file system buffer

Type: CONCEPT

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- File data is streamed directly from the operating system's file system buffer to the network socket.
- Node.js traditionally copies data into the V8 engine's managed heap as a Buffer or string.
