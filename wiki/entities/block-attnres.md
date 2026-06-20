---
type: entity
title: Block AttnRes
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Block AttnRes

Type: CONCEPT

## From [ai-research-part1](/entities/ai-research-part1.md) (2026-06-08)
- A variant of Attention Residuals designed to address memory and communication overhead.
- Partitions layers into blocks and attends over block-level representations.
- Reduces memory footprint and communication from O(Ld) to O(Nd).
- One of the variants trained for scaling laws, with ≈ 8 blocks.
- Fits L = 1.870 × C−0.058.
- Confines hidden-state magnitude growth within each block, yielding a bounded periodic pattern.
- Partitions layers into N blocks B1, ..., BN.
- For sources in a completed earlier block Bn, all share the block-level key/value bn.
- Reduces cost from O(L2) to O(LN) compared to Full AttnRes.

## From [ai-research-part8-micro02](/entities/ai-research-part8-micro02.md) (2026-06-09)
- A variant of Attention Residuals engineered to reconcile deep expressivity with hardware constraints.
- Partitions layers into blocks and applies softmax attention across blocks.
- Reduces communication and memory footprint.
