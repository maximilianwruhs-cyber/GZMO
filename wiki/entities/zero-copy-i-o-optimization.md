---
type: entity
title: Zero-Copy I/O Optimization
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Zero-Copy I/O Optimization

Type: CONCEPT

## From [[the-gzmo-daemon-high-performance-bun-refactor|the-gzmo-daemon-high-performance-bun-refactor]] (2026-06-08)
- Achieved by replacing Node.js fs.promises abstractions.
- Leverages Linux io_uring construct.
- Eliminates buffer copying latency.
