---
type: entity
title: Bun.Transpiler API
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---




# Bun.Transpiler API

Type: TOOL

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02]] (2026-06-09)
- Exposed by Bun for developers building meta-frameworks, static site generators, or custom test runners.
- Allows programmatic transpilation of TypeScript or JSX into vanilla JavaScript.
- Supports 'ts', 'tsx', 'js', or 'jsx' loaders.
- Has a transformSync method for fast execution of small inputs.
- Features a scanImports method for AST-level dependency analysis.
- Requires instantiating a single instance and reusing it for performance.

## From [[drive-research-bun-typescript-performance-tips-micro02|drive-research-bun-typescript-performance-tips-micro02]] (2026-06-09)
- A JavaScript and TypeScript runtime.
- Written predominantly in Zig.
- Distinguished by aggressive, systems-level optimizations.
- Utilizes Apple's JavaScriptCore (JSC) engine.
- Natively executes .ts and .tsx files without an external compilation step.
- Introduced a native metaprogramming construct for macros.
- Exposes its internal transpiler via the Bun.Transpiler JavaScript API.
- Provides utilities like Bun.peek() for synchronous promise inspection.
- Supports modern TC39 Stage 3 decorator proposal.
- Has a native metaprogramming construct for macros.
- Returns a string literal representing the internal state of a promise ('fulfilled', 'pending', or 'rejected').
- Bun exposes its internal transpiler via this API.
- Allows programmatic transpilation of TypeScript or JSX.
- Supports transformSync and scanImports methods.
- Can be used to execute privileged operations at build time.
- High-Performance Foreign Function Interfaces.
- A low-level utility for inspecting promise status and value synchronously.
- Bypasses the event loop.
- Can be used for high-throughput caching layers or custom HTTP routers.

## From [[high-performance-typescript-execution-and-architec-part1-micro02|high-performance-typescript-execution-and-architec-part1-micro02]] (2026-06-09)
- Exposes Bun's internal transpiler.
- Allows programmatic transpilation of TypeScript or JSX.
- Requires specifying a loader (e.g., 'ts', 'tsx').
- Has a transformSync method for fast execution.
- Includes a scanImports method for dependency analysis.
- A JavaScript and TypeScript runtime.
- Written predominantly in Zig.
- Features aggressive, systems-level optimizations.
- Utilizes Apple's JavaScriptCore (JSC) engine.
- Natively executes .ts and .tsx files without external compilation.
- Has an internal transpiler written in Zig that strips type annotations.
- Exposes its internal transpiler via the Bun.Transpiler JavaScript API.
- Introduces Bun.peek() for synchronous promise inspection.
- Includes a native metaprogramming construct for macros.
- Supports Foreign Function Interfaces (bun:ffi).
