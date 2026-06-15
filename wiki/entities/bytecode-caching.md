---
type: entity
title: Bytecode caching
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Bytecode caching

Type: CONCEPT

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02]] (2026-06-09)
- Introduced by Bun.
- Similar to Java compiling to .class files.
- Bun's bundler can compile JavaScript and TypeScript into .jsc bytecode cache files.
- When executed with --bytecode flag, Bun generates a proprietary binary representation of the AST.
- Reduces startup latency by bypassing the parsing phase.
- Represents application of compilation theories to the JavaScript ecosystem.
