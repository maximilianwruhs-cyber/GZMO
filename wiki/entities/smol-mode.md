---
type: entity
title: smol mode
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# smol mode

Type: CONCEPT

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Memory Tuning (smol mode).
- Developers can declare smol = true within bunfig.toml (or utilize the --smol CLI flag).
- This directive drastically alters the internal heuristics of the JSC engine, configuring the garbage collector to execute far more frequently and maintain a highly constrained heap size.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Declared within bunfig.toml or via --smol CLI flag.
- Drastically alters internal heuristics of the JSC engine.
- Configures the garbage collector to execute more frequently.
- Maintains a highly constrained heap size.
- Incurs a marginal latency penalty.
