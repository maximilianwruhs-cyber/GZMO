---
type: entity
title: pipeline-parallel (layer-split) execution modes
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# pipeline-parallel (layer-split) execution modes

Type: CONCEPT

## From [[drive-research-cuda-graph-capture-failure-workarounds-micro03|drive-research-cuda-graph-capture-failure-workarounds-micro03]] (2026-06-09)
- Structural changes in PR #20463 disabled graph reuse entirely when these modes were active.
- Execution mode where structural changes in PR #20463 disabled graph reuse.
- Graph reuse was re-enabled for these configurations in subsequent framework releases.
