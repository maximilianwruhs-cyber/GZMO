---
type: entity
title: bun:sqlite
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# bun:sqlite

Type: TOOL

## From [[drive-research-license-and-native-binding-analysis|drive-research-license-and-native-binding-analysis]] (2026-06-08)
- A fast, highly optimized SQLite implementation natively embedded in the Bun runtime.
- Requires no installation or external dependencies.
- Classified as Green.

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Bun integrates a native SQLite client (bun:sqlite) directly into the runtime binary.
- Because the database driver is compiled alongside the runtime, it circumvents the Node-API boundary entirely, executing queries 3 to 6 times faster than popular libraries like better-sqlite3.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Bun integrates a native SQLite client directly into the runtime binary.
- Executes queries 3 to 6 times faster than popular libraries like better-sqlite3.
- Bun integrates a native SQLite client.
- Resolves FFI bottleneck with optimized Foreign Function Interface (bun:ffi).
- Implements bun:ffi natively within its core.
- Uses an embedded instance of TinyCC.
- Dynamically generates and JIT compiles C bindings.
- Represents native C pointers directly as JavaScript number primitives.
- Provides the CString class.
- Automatically extracts the underlying memory pointer of array buffers.
- Supports JSCallback for asynchronous invocation of TypeScript functions.
- FFI system supports thread-safe callbacks.
- Overarching design philosophy replaces fragmented userland libraries with native implementations.
- Achieves dramatic reductions in latency by bridging JavaScript APIs to OS-level system calls using Zig.
- Implements Bun.file() and Bun.write() APIs.
- Bypasses traditional blocking I/O on Linux using io_uring.
- Native HTTP server initialized via Bun.serve().
- Integrates a native SQLite client (bun:sqlite).
- Native Bun.password.hash() API implements Argon2id and bcrypt.
- Incorporates a robust internal DNS cache.
- Experimental Bun.dns.prefetch() API.
- Functions seamlessly as a zero-configuration runtime.
- bunfig.toml file acts as a central orchestrator.
- Can be configured with smol = true for memory tuning.
- Utilizes a Zig/JSC architecture.
- Has a cold start latency of 8ms to 15ms.
- Achieves an estimated 180,000 requests per second in synthetic benchmarks.
- Has 95% to 98% API compatibility with the Node.js ecosystem.
- bun install reduces operating system syscalls significantly.
- bun.lockb is its proprietary binary lockfile format.
- bun test operates as a native, Jest-compatible test runner.
- Provides a profound, verifiable advantage in the JavaScript runtime ecosystem.
