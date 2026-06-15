---
type: entity
title: tsconfig.json
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---




# tsconfig.json

Type: TOOL

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro02]] (2026-06-09)
- Configuration file for TypeScript.
- Must be tailored to align with Bun's internal logic.
- Specific options like 'module', 'moduleResolution', 'allowImportingTsExtensions', 'verbatimModuleSyntax', 'noEmit', and 'types' are recommended for Bun compatibility.
- The 'emitDecoratorMetadata' and 'experimentalDecorators' flags require explicit declaration in the local file for decorator metadata to work correctly in Bun.

## From [[high-performance-typescript-execution-and-architec-part1-micro02|high-performance-typescript-execution-and-architec-part1-micro02]] (2026-06-09)
- Configuration file for TypeScript.
- Must be tailored to align with Bun's internal logic.
- Specific compiler options are recommended for peak compatibility with Bun.
- Can be extended using the 'extends' directive.

## From [[prompt-agent-engineering-part5-micro05|prompt-agent-engineering-part5-micro05]] (2026-06-09)
- Used instead of bunfig.toml
- Supports 'moduleResolution': 'bundler' and 'module': 'ESNext'
- Recommended excerpt provided
