---
type: entity
title: install.exact = true
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# install.exact = true

Type: CONCEPT

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- For reproducible production environments, enforcing install.exact = true within bunfig.toml is a mandatory best practice; it strips caret (^) and tilde (~) ranges from package.json, ensuring deterministic builds and neutralizing the risk of semantic versioning drift breaking native TypeScript execution.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Enforcing this within bunfig.toml is a mandatory best practice.
- Strips caret (^) and tilde (~) ranges from package.json.
