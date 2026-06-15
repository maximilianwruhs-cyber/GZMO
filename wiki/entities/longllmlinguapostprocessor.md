---
type: entity
title: LongLLMLinguaPostprocessor
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# LongLLMLinguaPostprocessor

Type: TOOL

## From [[drive-research-token-efficient-bol-processing-architecture|drive-research-token-efficient-bol-processing-architecture]] (2026-06-08)
- A node within the LlamaIndex ecosystem.
- Enforces strict context budgets.
- Dynamically reorders and drops statistically redundant tokens before the BoL data hits the heavy-compute generator model.
