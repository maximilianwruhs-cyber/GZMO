---
type: entity
title: EXSpec
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# EXSpec

Type: TOOL

## From [[drive-research-erbandbreite-und-latenzengp-sse|drive-research-erbandbreite-und-latenzengpässe]] (2026-06-08)
- advanced scheduler that dynamically regroups sequences by accepted length into uniform micro-batches
- abandons strict matrix realignment entirely
- maintains a sliding pool of sequences
- dynamically regroups requests of identical accepted lengths into micro-batches on the fly
- bypasses padding overhead
- maintains significant throughput improvements even at high batch sizes

## From [[the-architecture-of-speculative-decoding-and-infer-part2-micro02|the-architecture-of-speculative-decoding-and-infer-part2-micro02]] (2026-06-09)
- An advanced scheduler that dynamically regroups sequences by accepted length into uniform micro-batches.
- Abandons strict matrix realignment entirely.
- Maintains a sliding pool of sequences and dynamically regroups requests.
