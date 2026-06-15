---
type: entity
title: backend sampling
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---


# backend sampling

Type: CONCEPT

## From [[drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02|drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02]] (2026-06-09)
- Experimental backend sampling is enabled via --backend-sampling / -bs.
- Offloading token selection to the accelerator eliminates sequential CPU-bound sampling bottlenecks.
- Protects parallel prefill speeds from host scheduler delays.
