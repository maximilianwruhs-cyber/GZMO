---
type: entity
title: Bun.file
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Bun.file

Type: TOOL

## From [[refactoring-gzmo-daemon-for-native-bun-high-perfor|refactoring-gzmo-daemon-for-native-bun-high-perfor]] (2026-06-08)
- Will replace fs and fs.promises.
- Used for zero-copy file reading.
- Used to get file content as text.

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Bun dismantles this pipeline by implementing the Bun.file() and Bun.write() APIs.
- When an engineer passes a Bun.file() reference directly into an HTTP Response object, Bun executes a true zero-copy operation.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Implemented by Bun.
- Can be passed directly into an HTTP Response object for zero-copy operations.
