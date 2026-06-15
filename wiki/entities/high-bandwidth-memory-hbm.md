---
type: entity
title: High Bandwidth Memory (HBM)
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# High Bandwidth Memory (HBM)

Type: SYSTEM

## From [[drive-research-erbandbreite-und-latenzengp-sse|drive-research-erbandbreite-und-latenzengpässe]] (2026-06-08)
- Parameter weight matrix must be loaded from HBM into computational cores.
- AI accelerators are starved for data while waiting for memory transfers from HBM.

## From [[drive-research-llamacpp-optimization-blueprint-micro02|drive-research-llamacpp-optimization-blueprint-micro02]] (2026-06-09)
- Flash Attention prevents the materialization of the massive attention matrix in HBM.
