---
type: entity
title: user space
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# user space

Type: CONCEPT

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Node.js traditionally copies data from the kernel space into user space.
- File data is never instantiated as a string or ArrayBuffer in the JavaScript user space.
