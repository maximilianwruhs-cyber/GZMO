---
type: entity
title: Bun
created: 2026-06-08
updated: 2026-06-10
sources: 9
tags: []
status: draft
gzmo_synthetic: true
---









# Bun

Type: SYSTEM

## From [bun-versus-nodejs-architectural-evaluation-for-b](/entities/bun-versus-nodejs-architectural-evaluation-for-b.md) (2026-06-08)
- Written from the ground up in the Zig programming language
- Leverages Apple's JavaScriptCore engine
- Hyper-optimized toolkit
- Achieves cold start times of 8 to 15 milliseconds
- Implements nearly 90% of its total runtime natively in Zig, C, and C++
- Zero-configuration TypeScript transpiler embedded directly within the Bun executable
- Native recursive filesystem watcher
- Significantly lower baseline memory usage than Node.js

## From [drive-research-optimizing-pi-coding-agent-performance](/entities/drive-research-optimizing-pi-coding-agent-performance.md) (2026-06-08)
- Used as the asynchronous event loop for the TypeScript Reference Engine.

## From [drive-research-tinyfolder-gzmo-architecture-analysis-product](/entities/drive-research-tinyfolder-gzmo-architecture-analysis-product.md) (2026-06-08)
- Used as a base for tinyFolder / GZMO.

## From [drive-research-bun-typescript-performance-tips-micro04](/entities/drive-research-bun-typescript-performance-tips-micro04.md) (2026-06-09)
- A fast all-in-one JavaScript runtime.
- Codebase is almost 90% native.
- Has features like Transpiler, FFI, Macros, Utils, and bunfig.toml.

## From [high-performance-typescript-execution-and-architec-part1-micro04](/entities/high-performance-typescript-execution-and-architec-part1-micro04.md) (2026-06-09)
- A fast all-in-one JavaScript runtime.
- Codebase is almost 90% native.
- Does not emit decorator metadata if tsconfig inherits configuration from another file.

## From [prompt-agent-engineering-part5-micro04](/entities/prompt-agent-engineering-part5-micro04.md) (2026-06-09)
- Used as a runtime environment instead of pure Node.js.
- Offers faster startup times and native TypeScript support.
- Can be used as a drop-in replacement for Node.js.

## From [prompt-agent-engineering-part5-micro05](/entities/prompt-agent-engineering-part5-micro05.md) (2026-06-09)
- Respects tsconfig.json
- Compiles/transpiles on-the-fly without a build step
- Used for syntax checking via programmatic call or transpilation attempt
- Has an internal transpiler (Bun.transpiler)
- Used for syntax check via `bunx tsc --noEmit`

## From [openclaw-deep-research-part10-micro07](/entities/openclaw-deep-research-part10-micro07.md) (2026-06-10)
- Supported for TypeScript execution (scripts, dev, tests)
- Used via bun <file.ts> or bunx <tool>

## From [prompt-agent-engineering-part7-micro06](/entities/prompt-agent-engineering-part7-micro06.md) (2026-06-10)
- Runtime used for maximum speed and native TypeScript support
- Acts as a drop-in replacement for Node.js
