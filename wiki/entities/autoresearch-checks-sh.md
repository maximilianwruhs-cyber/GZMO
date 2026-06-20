---
type: entity
title: autoresearch.checks.sh
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# autoresearch.checks.sh

Type: CONCEPT

## From [drive-research-optimizing-pi-coding-agent-performance](/entities/drive-research-optimizing-pi-coding-agent-performance.md) (2026-06-08)
- Optional correctness gate for pi-autoresearch.
- Runs after successful benchmarks.
- If test suite or linter returns non-zero exit code, proposed code change is aborted.
- Benchmark script for pi-autoresearch.
- Executes target paths and returns a single metric matching the structure METRIC name=number.
- Managed via local project files.
