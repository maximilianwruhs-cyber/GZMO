---
type: entity
title: JavaScriptCore
created: 2026-06-08
updated: 2026-06-10
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---





# JavaScriptCore

Type: SYSTEM

## From [[refactoring-gzmo-daemon-for-native-bun-high-perfor|refactoring-gzmo-daemon-for-native-bun-high-perfor]] (2026-06-08)
- The engine that bunfig.toml aims to fine-tune.

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02]] (2026-06-09)
- The engine powering the Safari browser.
- Used by Bun.
- Employs a multi-tier JIT architecture optimized for rapid startup.
- Has four tiers: LLInt, Baseline JIT, DFG JIT, and FTL JIT.

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Runtimes utilizing JavaScriptCore are inherently fast but can exhibit aggressive memory consumption footprints, claiming large swaths of RAM to sustain high execution speeds and JIT compilation.
- Relying on the FinalizationRegistry to trigger native cleanup callbacks is inherently dangerous, as V8 and JavaScriptCore garbage collection cycles are non-deterministic and provide no guarantees on execution timing.
- Bun's superiority is derived exclusively from a Zig-based, JavaScriptCore-powered foundation that aggressively minimizes startup latency, bypasses transpilation friction, and connects directly to OS-level primitives like io_uring.

## From [[drive-research-bun-file-parsing-dependency-shortlist-micro02|drive-research-bun-file-parsing-dependency-shortlist-micro02]] (2026-06-09)
- Heap where SheetJS synchronously loads XML payloads.
- Main thread that is blocked by synchronous JSON parsing.

## From [[high-performance-typescript-execution-and-architec-part1-micro03|high-performance-typescript-execution-and-architec-part1-micro03]] (2026-06-10)
- An engine that can exhibit aggressive memory consumption footprints.
- Used by Bun to power its runtime.
