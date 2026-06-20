---
type: entity
title: bunfig.toml
created: 2026-06-08
updated: 2026-06-10
sources: 8
tags: []
status: draft
gzmo_synthetic: true
---








# bunfig.toml

Type: SYSTEM

## From [gzmo-daemon-validation-audit-and-bun-migration-rep](/entities/gzmo-daemon-validation-audit-and-bun-migration-rep.md) (2026-06-08)
- Untracked file according to 'git status'.
- Exists with correct content.
- Contains 'smol = true' and 'install.exact = true'.

## From [refactoring-gzmo-daemon-for-native-bun-high-perfor](/entities/refactoring-gzmo-daemon-for-native-bun-high-perfor.md) (2026-06-08)
- A new configuration file to fine-tune the JavaScriptCore engine.
- Will introduce strict configuration.
- Contains settings like 'smol = true' and '[install] exact = true'.

## From [drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03](/entities/drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03.md) (2026-06-09)
- Environmental Control and Optimization via bunfig.toml.
- The bunfig.toml file, placed at the project root, acts as the central orchestrator for runtime heuristics, test execution, and dependency resolution.
- Developers can declare smol = true within bunfig.toml (or utilize the --smol CLI flag).
- Enforcing install.exact = true within bunfig.toml is a mandatory best practice.
- Engineers can set rigid coverageThreshold metrics (e.g., forcing 90% statement coverage) to fail CI pipelines automatically.
- Mastering specific finesse techniques—such as intentionally bypassing the asynchronous event loop with Bun.peek(), eliminating runtime I/O latency via compile-time execution macros, utilizing the JIT-compiled bun:ffi for memory-safe native execution across language boundaries, and strictly managing memory footprints and dependency locks via bunfig.toml—engineers can construct heavily optimized, type-safe architectures.

## From [drive-research-bun-typescript-performance-tips-micro03](/entities/drive-research-bun-typescript-performance-tips-micro03.md) (2026-06-09)
- File placed at the project root.
- Acts as the central orchestrator for runtime heuristics, test execution, and dependency resolution.
- Can declare smol = true for memory tuning.
- Can enforce install.exact = true for deterministic builds.
- Contains the [test] configuration block.
- Used for managing memory footprints and dependency locks.
- A configuration format mentioned alongside Ini.

## From [drive-research-bun-typescript-performance-tips-micro04](/entities/drive-research-bun-typescript-performance-tips-micro04.md) (2026-06-09)
- Configuration file for Bun.
- Documentation available on Bun.sh and Bun.com.

## From [high-performance-typescript-execution-and-architec-part1-micro04](/entities/high-performance-typescript-execution-and-architec-part1-micro04.md) (2026-06-09)
- Configuration file for Bun.
- Mentioned in Bun documentation.

## From [prompt-agent-engineering-part5-micro05](/entities/prompt-agent-engineering-part5-micro05.md) (2026-06-09)
- Mostly superfluous when using tsconfig.json
- Used for very specific overrides like hot-reload-paths

## From [high-performance-typescript-execution-and-architec-part1-micro03](/entities/high-performance-typescript-execution-and-architec-part1-micro03.md) (2026-06-10)
- Acts as the central orchestrator for runtime heuristics, test execution, and dependency resolution.
- Used to declare 'smol = true' for memory tuning.
