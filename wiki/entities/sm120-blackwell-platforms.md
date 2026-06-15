---
type: entity
title: SM120 Blackwell platforms
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# SM120 Blackwell platforms

Type: SYSTEM

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- FP8 QuartzNet networks experience up to a 77% performance regression under TensorRT 10.13.2.
- This regression can be worked around by forcing the auxiliary execution stream count to zero (--maxAuxStreams=0).
