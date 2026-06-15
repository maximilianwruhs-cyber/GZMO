---
type: entity
title: JavaScriptCore (JSC)
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# JavaScriptCore (JSC)

Type: SYSTEM

## From [[drive-research-license-and-native-binding-analysis|drive-research-license-and-native-binding-analysis]] (2026-06-08)
- The JavaScript engine utilized by the Bun runtime.
- Native modules relying on V8-specific paradigms will fail to compile or execute.
- Edge cases involving asynchronous thread-safe functions frequently result in segmentation faults within this environment.

## From [[drive-research-bun-typescript-performance-tips-micro02|drive-research-bun-typescript-performance-tips-micro02]] (2026-06-09)
- The engine powering the Safari browser.
- Bun utilizes JSC.
- Employs a multi-tier JIT architecture optimized for rapid startup.
- Has a low memory footprint.

## From [[high-performance-typescript-execution-and-architec-part1-micro02|high-performance-typescript-execution-and-architec-part1-micro02]] (2026-06-09)
- The JavaScript engine powering the Safari browser.
- Utilized by Bun.
- Employs a multi-tier JIT architecture optimized for rapid startup.
- Has a low memory footprint.
- Includes LLInt, Baseline JIT, DFG JIT, and FTL JIT compilation strategies.
