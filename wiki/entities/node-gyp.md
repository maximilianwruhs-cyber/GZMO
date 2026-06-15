---
type: entity
title: node-gyp
created: 2026-06-08
updated: 2026-06-09
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---





# node-gyp

Type: TOOL

## From [[drive-research-license-and-native-binding-analysis|drive-research-license-and-native-binding-analysis]] (2026-06-08)
- A tool utilized by Bun when encountering packages with native bindings.
- Necessitates a strict chain of environmental prerequisites on Ubuntu.
- Strict reliance on local host compilation using node-gyp classifies packages as Red.

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Historically, the Node.js ecosystem relied on Node-API (N-API) or node-gyp native addons.
- Legacy packages often rely on deeply rooted node-gyp compilation steps.

## From [[drive-research-bun-file-parsing-dependency-shortlist-micro01|drive-research-bun-file-parsing-dependency-shortlist-micro01]] (2026-06-09)
- used for compiling C/C++ native bindings
- dependencies relying on it are excluded in favor of pure JavaScript or TypeScript implementations for Bun

## From [[drive-research-bun-file-parsing-dependency-shortlist-micro02|drive-research-bun-file-parsing-dependency-shortlist-micro02]] (2026-06-09)
- Tool used by packages to compile native C++ code during npm install.
- Native modules utilizing it frequently fail to compile or cause issues when migrating to alternative runtimes like Bun or Deno.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Used by Node.js ecosystem for native addons.
- Legacy packages may rely on node-gyp compilation steps.
