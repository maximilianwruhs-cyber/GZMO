---
type: entity
title: -b (Logical Batch)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# -b (Logical Batch)

Type: TOOL

## From [[optimizing-nvidia-blackwell-sm120-part1-micro02|optimizing-nvidia-blackwell-sm120-part1-micro02]] (2026-06-09)
- Tokens processed simultaneously.
- Higher values heavily improve GPU utilization during prompt ingestion, but increase the time-to-first-token (TTFT).
- Defines the maximum number of tokens processed simultaneously during the pipeline evaluation phase.
