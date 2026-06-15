---
type: entity
title: 'perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp'
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp

Type: CONCEPT

## From [[drive-research-cuda-graph-capture-failure-workarounds-micro03|drive-research-cuda-graph-capture-failure-workarounds-micro03]] (2026-06-09)
- Enabled (ON) in Build 8289 (Baseline).
- Disabled (OFF) in Build 8367 (PR #20463).
- Enabled (ON) in Build 8367 (Reverted).
- Re-enabled for pipeline-parallel configurations in subsequent framework releases.
- Highly sensitive to peer-to-peer transport latency and synchronization of host-staged transfers.
- GitHub commit related to re-enabling graph reuse.
- GitHub issue detailing performance regression due to graph reuse disable.
