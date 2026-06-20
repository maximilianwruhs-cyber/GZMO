---
type: entity
title: Programmatic Dependency Launch (PDL)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Programmatic Dependency Launch (PDL)

Type: CONCEPT

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01.md) (2026-06-09)
- A mechanism used by deep learning frameworks to pipeline and overlap successive GPU kernels.
- Host-side runtime queries the device for PDL capability, which is true for devices with a major compute capability of 9 or higher.
- Permits the CUDA driver to dispatch dependent kernels before preceding kernels have completed execution.
